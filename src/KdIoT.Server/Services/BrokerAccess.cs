using System.Text;
using Google.Protobuf;
using KdIoT.Server.Data;
using Microsoft.EntityFrameworkCore;
using RabbitMQ.Client;
using RabbitMQ.Client.Events;

namespace KdIoT.Server.Services {

    public class BrokerAccessService : IHostedService, IDisposable {
        private readonly ILogger<BrokerAccessService> _logger;
        //private readonly AppDbContext _appDbContext;
        private readonly IServiceProvider _provider;
        ConnectionFactory _factory;
        IConnection? _connection;
        IModel? _channel;

        CancellationTokenSource? _taskstoppingTokenSource;
        AsyncEventingBasicConsumer? _consumer;

        public BrokerAccessService(ILogger<BrokerAccessService> logger, IServiceProvider provider, IServiceScope scope) {
            _logger = logger;
            _scope = scope;
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

            _channel.QueueDeclare(queue: "ServerQueue",
                     durable: false,
                     exclusive: false,
                     autoDelete: false,
                     arguments: null);

            _consumer = new AsyncEventingBasicConsumer(_channel);
            _consumer.Received += MessageRecived;
            _channel.QueueBind("ServerQueue", "amq.topic", "iot.*.sendtoserver");
            _channel.BasicConsume(queue: "ServerQueue",
                                     autoAck: true,
                                     consumer: _consumer);

            var task = Task.Run(async () => await DoWork(_taskstoppingTokenSource.Token).ConfigureAwait(false)).ConfigureAwait(false);
        }

        private async Task MessageRecived(object model, BasicDeliverEventArgs ea) {
            var body = ea.Body.ToArray();
            var message = ProtoBrokerMsgs.IoTMessage.Parser.ParseFrom(body);
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

            Telemetry tel = new Telemetry { Device = device };
            await dbContext.Telemetries.AddAsync(tel);
            await dbContext.SaveChangesAsync();
            Console.WriteLine("Saved");

        }

        public void SendSwitch(string id_device) {
            var message = new ProtoBrokerMsgs.ServerMessage {
                Command = ProtoBrokerMsgs.ServerMessage.Types.Cmd.Switch,
                Body = "text"
            };

            _channel.BasicPublish(exchange: "amq.topic",
                                routingKey: $"iot.{id_device}.receive",
                                basicProperties: null,
                                body: message.ToByteArray());
        }

        public void SendGlobalSwitch() {
            var message = new ProtoBrokerMsgs.ServerMessage {
                Command = ProtoBrokerMsgs.ServerMessage.Types.Cmd.Switch,
                Body = "text"
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
    }

}