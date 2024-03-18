using System.Text.Json;
using KdIoT.Server.Data;
using KdIoT.Server.Services;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using NodaTime;
using NodaTime.Extensions;
using System.Web;
using System.Net;

namespace KdIoT.Server.Controllers {

    [ApiController]
    [Route("api")]
    public class IoTManagerController : ControllerBase {

        private readonly AppDbContext _appDbContext;
        private BrokerAccessService _brokerAccessService;
        private SystemStatusService _systemStatusService;

        public IoTManagerController(AppDbContext appDbContext, BrokerAccessService brokerAccessService, SystemStatusService systemStatusService) {
            _appDbContext = appDbContext;
            _brokerAccessService = brokerAccessService;
            _systemStatusService = systemStatusService;
        }


        [HttpGet("device/{deviceName}/[action]")]
        [ProducesResponseType(typeof(TelemetryDto), StatusCodes.Status200OK)]
        public async Task<IActionResult> LastMeassure([FromRoute] string deviceName) {
            var result = await _appDbContext.Telemetries.AsQueryable()
                .Include(x => x.Device)
                .Where(c => c.Device.DeviceName.Equals(deviceName.ToLower()))
                .OrderByDescending(c => c.MeasuredTime)
                .ThenByDescending(c => c.SubmitedTime)
                .FirstOrDefaultAsync();
            
            if(result is null)
                return NoContent();
            
            
            return Ok(new TelemetryDto(result.Temperature, result.Pressure, result.Humidity, result.SubmitedTime.ToDateTimeUtc(), result.MeasuredTime.ToDateTimeUtc()));
        }

        [HttpGet("device/{deviceName}/[action]")]
        public async Task<IActionResult> LastDayAverageMeassure([FromRoute] string deviceName) {
            var now = SystemClock.Instance.GetCurrentInstant();
            var toLastDay = now.Minus(Duration.FromDays(1));
            
            //FromSql($"SELECT AVG(Temperature), AVG(Humidity), AVG(Pressure) FROM Telemetries LEFT JOIN Devices ON Telemetries.Device = Devices.DeviceId
            // WHERE Devices.DeviceName = {deviceName} AND Telemetries.MeasuredTime...")

            var result = await _appDbContext.Telemetries.AsQueryable()
                .Include(x => x.Device)
                .Where(c => c.Device.DeviceName.Equals(deviceName.ToLower()))
                .Where(c => c.MeasuredTime > toLastDay)
                .GroupBy(c => c.Device)
                .Select(g => new {Temperature = g.Average(c => c.Temperature), Humidity = g.Average(c => c.Humidity), Pressure = g.Average(c => c.Pressure)})
                .ToListAsync();


            return Ok(result);
        }


        [HttpGet("device/{deviceName}/[action]")]
        public async Task<IEnumerable<TelemetryDto>> LastDayMeassures([FromRoute] string deviceName) {
            var now = SystemClock.Instance.GetCurrentInstant();
            var toLastDay = now.Minus(Duration.FromDays(1));
            

            var result = await _appDbContext.Telemetries.AsQueryable()
                .Include(x => x.Device)
                .Where(c => c.Device.DeviceName.Equals(deviceName.ToLower()))
                .Where(c => c.MeasuredTime > toLastDay)
                .OrderByDescending(c => c.MeasuredTime)
                .ThenByDescending(c => c.SubmitedTime)
                .ToListAsync();

            var response = result.Select(s => new TelemetryDto(s.Temperature, s.Humidity, s.Pressure, s.SubmitedTime.ToDateTimeUtc(), s.MeasuredTime.ToDateTimeUtc()));
            return response;
        }

        [HttpGet("[action]")]
        [ProducesResponseType(typeof(Dictionary<string, DateTime>), StatusCodes.Status200OK)]
        public IActionResult DeviceActivityTable() {
            Dictionary<string, DateTime> table = _systemStatusService.GetAllLastSeen();
            return Ok(table);
        }


        /////////----
        
        [HttpGet("device/{deviceName}/[action]")]
        public void SendSwitch([FromRoute] string deviceName) {
            _brokerAccessService.SendSwitch(deviceName.ToLower());
        }

        [HttpGet("[action]")]
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

        [HttpGet("[action]")]
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

        public record struct TelemetryDto(float Temperature, float Humidity, float Pressure, DateTime SubmitedTime, DateTime MeasuredTime);
    }
}
