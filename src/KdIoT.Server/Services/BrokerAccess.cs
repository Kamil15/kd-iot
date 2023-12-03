using System.Text;
using RabbitMQ.Client;
using RabbitMQ.Client.Events;

namespace KdIoT.Server.Services {

    public class BrokerAccessService : IHostedService, IDisposable {
        private readonly ILogger<BrokerAccessService> _logger;
        ConnectionFactory? _factory;
        IConnection _connection;
        IModel? _channel;

        CancellationTokenSource? _taskstoppingTokenSource;
        AsyncEventingBasicConsumer? _consumer;

        public BrokerAccessService(ILogger<BrokerAccessService> logger) {
            _logger = logger;
            _factory = new ConnectionFactory {
                HostName = "rabbitmq",
                UserName = "iotdevice",
                Password = "IttrulyisanioTdevice",
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
            _consumer.Received += Recived;
            _channel.QueueBind("ServerQueue", "amq.topic", "ServerRoute");
            _channel.BasicConsume(queue: "ServerQueue",
                                     autoAck: true,
                                     consumer: _consumer);

            var task = Task.Run(async () => await DoWork(_taskstoppingTokenSource.Token).ConfigureAwait(false)).ConfigureAwait(false);
        }

        private async Task Recived(object model, BasicDeliverEventArgs ea) {
            
            var body = ea.Body.ToArray();
            var message = Encoding.UTF8.GetString(body);
            _logger.LogInformation($" [x] Received {message}, ea.ConsumerTag: {ea.ConsumerTag}, ea.Exchange: {ea.Exchange}");
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