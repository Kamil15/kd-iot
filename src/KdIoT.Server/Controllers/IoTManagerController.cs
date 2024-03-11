using KdIoT.Server.Data;
using KdIoT.Server.Services;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;

namespace KdIoT.Server.Controllers {

    [ApiController]
    [Route("[controller]/[action]")]
    public class IoTManagerController : ControllerBase {

        private readonly AppDbContext _appDbContext;
        private BrokerAccessService _brokerAccessService;

        public IoTManagerController(AppDbContext appDbContext, BrokerAccessService brokerAccessService) {
            _appDbContext = appDbContext;
            _brokerAccessService = brokerAccessService;
        }

        [HttpGet]
        public async Task<Telemetry> LastMeassure([FromBody] string deviceId) {
            var result = await _appDbContext.Telemetries.AsQueryable()
                .OrderByDescending(c => c.MeasuredTime)
                .ThenByDescending(c => c.SubmitedTime).FirstOrDefaultAsync();
            
            
            return result!;
        }
        
        [HttpGet]
        public void SendSwitch([FromBody] string deviceId) {
            _brokerAccessService.SendSwitch("air");
        }

        [HttpGet]
        public void SendGlobalSwitch() {
            _brokerAccessService.SendGlobalSwitch();
        }

        /////////----

        public record WeatherForecast(DateOnly Date, int TemperatureC, string? Summary) {
            public int TemperatureF => 32 + (int)(TemperatureC / 0.5556);
        }

        string[] summaries = new[] {
            "Freezing", "Bracing", "Chilly", "Cool", "Mild", "Warm", "Balmy", "Hot", "Sweltering", "Scorching"
        };

        [HttpGet]
        [ProducesResponseType(StatusCodes.Status200OK)]
        public WeatherForecast[] GetTheThing() {
            var forecast = Enumerable.Range(1, 5).Select(index =>
                new WeatherForecast(
                    DateOnly.FromDateTime(DateTime.Now.AddDays(index)),
                    Random.Shared.Next(-20, 55),
                    summaries[Random.Shared.Next(summaries.Length)]
                ))
                .ToArray();
            return forecast;

        }
    }
}
