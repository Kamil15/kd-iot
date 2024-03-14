using System.Text.Json;
using KdIoT.Server.Data;
using KdIoT.Server.Services;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using NodaTime.Extensions;

namespace KdIoT.Server.Controllers {

    [ApiController]
    [Route("api/[controller]/[action]")]
    public class IoTManagerController : ControllerBase {

        private readonly AppDbContext _appDbContext;
        private BrokerAccessService _brokerAccessService;
        private SystemStatusService _systemStatusService;

        public IoTManagerController(AppDbContext appDbContext, BrokerAccessService brokerAccessService, SystemStatusService systemStatusService) {
            _appDbContext = appDbContext;
            _brokerAccessService = brokerAccessService;
            _systemStatusService = systemStatusService;
        }

        [HttpGet]
        public async Task<Telemetry> LastMeassure([FromQuery] string deviceName) {
            var result = await _appDbContext.Telemetries.AsQueryable()
                .Include(x => x.Device)
                .Where(c => c.Device.DeviceName.Equals(deviceName.ToLower()))
                .OrderByDescending(c => c.MeasuredTime)
                .ThenByDescending(c => c.SubmitedTime)
                .FirstOrDefaultAsync();
            
            
            return result!;
        }

        [HttpGet]
        public async Task<List<Telemetry>> LastDayMeassure([FromQuery] string deviceName) {
            var now = DateTime.Now;
            var toLastDay = now.Subtract(TimeSpan.FromDays(1)).ToInstant();
            

            var result = await _appDbContext.Telemetries.AsQueryable()
                .Include(x => x.Device)
                .Where(c => c.Device.DeviceName.Equals(deviceName.ToLower()))
                .Where(c => c.MeasuredTime > toLastDay)
                .OrderByDescending(c => c.MeasuredTime)
                .ThenByDescending(c => c.SubmitedTime)
                .ToListAsync();

            return result!;
        }

        [HttpGet]
        public async Task<string> LastDayAverageMeassure([FromQuery] string deviceName) {
            var now = DateTime.Now;
            var toLastDay = now.Subtract(TimeSpan.FromDays(1)).ToInstant();
            
            //FromSql($"SELECT AVG(Temperature), AVG(Humidity), AVG(Pressure) FROM Telemetries LEFT JOIN Devices ON Telemetries.Device = Devices.DeviceId
            // WHERE Devices.DeviceName = {deviceName} AND Telemetries.MeasuredTime...")

            var result = await _appDbContext.Telemetries.AsQueryable()
                .Include(x => x.Device)
                .Where(c => c.Device.DeviceName.Equals(deviceName.ToLower()))
                .Where(c => c.MeasuredTime > toLastDay)
                .GroupBy(c => c)
                .Select(g => new {Humidity = g.Average(c => c.Humidity), Pressure = g.Average(c => c.Pressure), Temperature = g.Average(c => c.Temperature)})
                .ToListAsync();

            

            //var a = from p in _appDbContext.Telemetries
            //select new {};

            return JsonSerializer.Serialize(result);
        }

        [HttpGet]
        public string DeviceActivityTable() {
            List<(string, DateTime)> table = _systemStatusService.GetAllLastSeen();
            return JsonSerializer.Serialize(table);
        }




        /////////----
        
        [HttpGet]
        public void SendSwitch([FromQuery] string deviceName) {
            _brokerAccessService.SendSwitch(deviceName.ToLower());
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
