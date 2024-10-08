using System.Text;
using Google.Protobuf;
using KdIoT.Server.Data;
using Microsoft.EntityFrameworkCore;
using NodaTime;
using NodaTime.Extensions;
using RabbitMQ.Client;
using RabbitMQ.Client.Events;

namespace KdIoT.Server.Services {

    public class BrokerAccessService : IHostedService, IDisposable {
        private readonly ILogger<BrokerAccessService> _logger;
        private readonly SystemStatusService _systemStatusService;

        //private readonly AppDbContext _appDbContext;
        private readonly IServiceProvider _provider;
        ConnectionFactory _factory;
        IConnection? _connection;
        IModel? _channel;

        CancellationTokenSource? _taskstoppingTokenSource;
        AsyncEventingBasicConsumer? _consumerTelemetry;
        AsyncEventingBasicConsumer? _consumerActivity;

        public BrokerAccessService(ILogger<BrokerAccessService> logger, SystemStatusService systemStatusService, IServiceProvider provider) {
            _logger = logger;
            _systemStatusService = systemStatusService;
            _provider = provider;
            _factory = new ConnectionFactory {
                HostName = "rabbitmq",
                UserName = "theserver",
                Password = "myserverpass",
                DispatchConsumersAsync = true
                //Port = 5671
            };
            //_factory.Ssl.Enabled = true;
        }

        public async Task StartAsync(CancellationToken stoppingToken) {
            await Task.Delay(TimeSpan.FromSeconds(5));
            _logger.LogInformation("Timed Hosted Service running.");
            _taskstoppingTokenSource = new CancellationTokenSource();


            _connection = _factory.CreateConnection();
            _channel = _connection.CreateModel();

            _channel.QueueDeclare(queue: "ServerQueueTelemetry",
                     durable: false,
                     exclusive: false,
                     autoDelete: false,
                     arguments: null);
            
            _channel.QueueDeclare(queue: "ServerQueueActivity",
                     durable: false,
                     exclusive: false,
                     autoDelete: false,
                     arguments: null);

            _consumerTelemetry = new AsyncEventingBasicConsumer(_channel);
            _consumerTelemetry.Received += TelemetryMessageRecived;
            _channel.QueueBind("ServerQueueTelemetry", "amq.topic", "iotserver.*.sendtelemetry");
            _channel.BasicConsume(queue: "ServerQueueTelemetry",
                                     autoAck: true,
                                     consumer: _consumerTelemetry);

            _consumerActivity = new AsyncEventingBasicConsumer(_channel);
            _consumerActivity.Received += ActivityMessageRecived;
            _channel.QueueBind("ServerQueueActivity", "amq.topic", "iotserver.*.sendactivity");
            _channel.BasicConsume(queue: "ServerQueueActivity",
                                     autoAck: true,
                                     consumer: _consumerActivity);

            var task = Task.Run(async () => await DoWork(_taskstoppingTokenSource.Token).ConfigureAwait(false)).ConfigureAwait(false);
        }

        private async Task TelemetryMessageRecived(object model, BasicDeliverEventArgs ea) {
            var body = ea.Body.ToArray();
            var message = ProtoBrokerMsgs.TelemetryMessage.Parser.ParseFrom(body);
            //_logger.LogInformation($" [x] [ea.DeliveryTag:] {ea.DeliveryTag}, [ea.ConsumerTag:] {ea.ConsumerTag}, [ea.Exchange:] {ea.Exchange}" +
            //$" [x] [ea.Redelivered:] {ea.Redelivered}, [ea.RoutingKey:] {ea.RoutingKey}, [ea.BasicProperties.UserId:] {ea.BasicProperties.ReplyTo}" +
            //$" [x] message.Pressure: {message.Pressure}, message.Humidity: {message.Humidity}, message.Temperature: {message.Temperature}");



            var idDeviceFromRoutingKey = ea.RoutingKey.Split('.')[1];
            if (!StringComparer.CurrentCultureIgnoreCase.Equals(idDeviceFromRoutingKey, message.IdDevice)) {
                _logger.LogInformation(
                    $"idDevice from RoutingKey and from Message are incorrect, idDeviceFromRoutingKey: {idDeviceFromRoutingKey} | message.IdDevice: {message.IdDevice}");
                return;
            }

            using var scope = _provider.CreateScope();
            using var dbContext = scope.ServiceProvider.GetRequiredService<AppDbContext>();


            var device = await dbContext.Devices.AsQueryable().FirstOrDefaultAsync(f => f.DeviceName.Equals(message.IdDevice.ToLower()));

            if (device is null) {
                device = new Device { DeviceName = message.IdDevice.ToLower() };
                //await dbContext.Devices.AddAsync(device);
            }

            Telemetry Telemetry = new Telemetry {
                Device = device,
                Humidity = message.Humidity,
                Temperature = message.Temperature,
                Pressure = message.Pressure,
                MeasuredTime = message.Timestamp.ToDateTime().ToInstant(),
                SubmitedTime = SystemClock.Instance.GetCurrentInstant(),
            };

            
            await dbContext.Telemetries.AddAsync(Telemetry);
            await dbContext.SaveChangesAsync();

            _systemStatusService.UpdateLastSeen(message.IdDevice.ToLower(), DateTime.Now);

        }

        private async Task ActivityMessageRecived(object model, BasicDeliverEventArgs ea) {
            var body = ea.Body.ToArray();
            var message = ProtoBrokerMsgs.ActivityMesssage.Parser.ParseFrom(body);
            var idDeviceFromRoutingKey = ea.RoutingKey.Split('.')[1];
            if (!StringComparer.CurrentCultureIgnoreCase.Equals(idDeviceFromRoutingKey, message.IdDevice)) {
                _logger.LogInformation(
                    $"idDevice from RoutingKey and from Message are incorrect, idDeviceFromRoutingKey: {idDeviceFromRoutingKey} | message.IdDevice: {message.IdDevice}");
                return;
            }

            await Task.Yield(); //just to surpass some warring

            _systemStatusService.UpdateLastSeen(message.IdDevice.ToLower(), DateTime.Now);
        }

        public void SendSwitch(string id_device, SwitchStates state) {
            var typestate = state switch {
                SwitchStates.Switch => ProtoBrokerMsgs.ServerMessage.Types.Cmd.Switch,
                SwitchStates.Check => ProtoBrokerMsgs.ServerMessage.Types.Cmd.Check,
                SwitchStates.Uncheck => ProtoBrokerMsgs.ServerMessage.Types.Cmd.Uncheck,
                _ => ProtoBrokerMsgs.ServerMessage.Types.Cmd.Switch,
            };

            var message = new ProtoBrokerMsgs.ServerMessage {
                Command = typestate
            };

            var routingDeviceId = id_device.Replace(".", String.Empty).ToLower();

            _channel.BasicPublish(exchange: "amq.topic",
                                routingKey: $"iot.{routingDeviceId}.receive",
                                basicProperties: null,
                                body: message.ToByteArray());
        }

        public void SendGlobalSwitch(SwitchStates state) {
            var typestate = state switch {
                SwitchStates.Switch => ProtoBrokerMsgs.ServerMessage.Types.Cmd.Switch,
                SwitchStates.Check => ProtoBrokerMsgs.ServerMessage.Types.Cmd.Check,
                SwitchStates.Uncheck => ProtoBrokerMsgs.ServerMessage.Types.Cmd.Uncheck,
                _ => ProtoBrokerMsgs.ServerMessage.Types.Cmd.Switch,
            };

            var message = new ProtoBrokerMsgs.ServerMessage {
                Command = typestate
            };

            _channel.BasicPublish(exchange: "amq.topic",
                                routingKey: $"iot.global",
                                basicProperties: null,
                                body: message.ToByteArray());
        }

        private async Task DoWork(CancellationToken stoppingToken) {
            while (!stoppingToken.IsCancellationRequested) {
                await Task.Delay(TimeSpan.FromSeconds(15), stoppingToken);
            }
            _logger.LogInformation("DoWork IsCancellationRequested");
        }

        public Task StopAsync(CancellationToken stoppingToken) {
            _taskstoppingTokenSource?.Cancel();

            return Task.CompletedTask;
        }

        public void Dispose() {
            _connection?.Dispose();
            _channel?.Dispose();
            _taskstoppingTokenSource?.Dispose();
        }


        public enum SwitchStates {
            Switch,
            Check,
            Uncheck
        }
    }

}