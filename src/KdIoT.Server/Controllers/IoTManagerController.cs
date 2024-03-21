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
        public async Task<IActionResult> LastMeasure([FromRoute] string deviceName) {
            var result = await _appDbContext.Telemetries.AsQueryable()
                .Include(x => x.Device)
                .Where(c => c.Device.DeviceName.Equals(deviceName.ToLower()))
                .OrderByDescending(c => c.MeasuredTime)
                .ThenByDescending(c => c.SubmitedTime)
                .FirstOrDefaultAsync();
            
            if(result is null)
                return NoContent();
            
            //return Ok(new {result.Temperature, result.Pressure, result.Humidity, submit = result.SubmitedTime.InUtc().ToString(), measured = result.MeasuredTime.InUtc().ToString()});
            return Ok(new TelemetryDto(result.Temperature, result.Humidity, result.Pressure, result.SubmitedTime.ToDateTimeUtc(), result.MeasuredTime.ToDateTimeUtc()));
        }

        [HttpGet("device/{deviceName}/[action]")]
        [ProducesResponseType(typeof(IEnumerable<TelemetryDto>), StatusCodes.Status200OK)]
        public async Task<IActionResult> LastDayMeasures([FromRoute] string deviceName) {
            var now = SystemClock.Instance.GetCurrentInstant();
            var toLastDay = now.Minus(Duration.FromDays(1));
            

            var result = await _appDbContext.Telemetries.AsQueryable()
                .Include(x => x.Device)
                .Where(c => c.Device.DeviceName.Equals(deviceName.ToLower()))
                .Where(c => c.MeasuredTime >= toLastDay)
                .OrderByDescending(c => c.MeasuredTime)
                .ThenByDescending(c => c.SubmitedTime)
                .ToListAsync();

            var response = result.Select(s => new TelemetryDto(s.Temperature, s.Humidity, s.Pressure, s.SubmitedTime.ToDateTimeUtc(), s.MeasuredTime.ToDateTimeUtc()));
            return Ok(response);
        }

        [HttpGet("device/{deviceName}/[action]")]
        public async Task<IActionResult> LastDayAverageMeasure([FromRoute] string deviceName) {
            var now = SystemClock.Instance.GetCurrentInstant();
            var toLastDay = now.Minus(Duration.FromDays(1));
            
            //FromSql($"SELECT AVG(Temperature), AVG(Humidity), AVG(Pressure) FROM Telemetries LEFT JOIN Devices ON Telemetries.Device = Devices.DeviceId
            // WHERE Devices.DeviceName = {deviceName} AND Telemetries.MeasuredTime...")

            var result = await _appDbContext.Telemetries.AsQueryable()
                .Include(x => x.Device)
                .Where(c => c.Device.DeviceName.Equals(deviceName.ToLower()))
                .Where(c => c.MeasuredTime >= toLastDay)
                .GroupBy(c => c.Device)
                .Select(g => new {Temperature = g.Average(c => c.Temperature), Humidity = g.Average(c => c.Humidity), Pressure = g.Average(c => c.Pressure)})
                .ToListAsync();


            return Ok(result);
        }

        [HttpGet("device/{deviceName}/[action]")]
        public async Task<IActionResult> LastAverageMeasure([FromRoute] string deviceName, [FromQuery] long seconds) {
            var now = SystemClock.Instance.GetCurrentInstant();
            var toLastDay = now.Minus(Duration.FromSeconds(seconds));
            
            //FromSql($"SELECT AVG(Temperature), AVG(Humidity), AVG(Pressure) FROM Telemetries LEFT JOIN Devices ON Telemetries.Device = Devices.DeviceId
            // WHERE Devices.DeviceName = {deviceName} AND Telemetries.MeasuredTime...")

            var result = await _appDbContext.Telemetries.AsQueryable()
                .Include(x => x.Device)
                .Where(c => c.Device.DeviceName.Equals(deviceName.ToLower()))
                .Where(c => c.MeasuredTime >= toLastDay)
                .GroupBy(c => c.Device)
                .Select(g => new {Temperature = g.Average(c => c.Temperature), Humidity = g.Average(c => c.Humidity), Pressure = g.Average(c => c.Pressure)})
                .ToListAsync();


            return Ok(result);
        }

        [HttpGet("device/{deviceName}/[action]")]
        public async Task<IActionResult> AverageMeasureFromDate([FromRoute] string deviceName, [FromQuery] DateTime from) {
            var now = SystemClock.Instance.GetCurrentInstant();
            var fromLastDate = now.Minus(now - from.ToInstant());

            //FromSql($"SELECT AVG(Temperature), AVG(Humidity), AVG(Pressure) FROM Telemetries LEFT JOIN Devices ON Telemetries.Device = Devices.DeviceId
            // WHERE Devices.DeviceName = {deviceName} AND Telemetries.MeasuredTime...")

            var result = await _appDbContext.Telemetries.AsQueryable()
                .Include(x => x.Device)
                .Where(c => c.Device.DeviceName.Equals(deviceName.ToLower()))
                .Where(c => c.MeasuredTime >= fromLastDate)
                .GroupBy(c => c.Device)
                .Select(g => new {Temperature = g.Average(c => c.Temperature), Humidity = g.Average(c => c.Humidity), Pressure = g.Average(c => c.Pressure)})
                .ToListAsync();


            return Ok(result);
        }

        [HttpGet("device/{deviceName}/[action]")]
        public async Task<IActionResult> AverageMeasureFromToDate([FromRoute] string deviceName, [FromQuery] DateTime from, [FromQuery] DateTime to) {
            var toDate = to.ToInstant();
            var fromLastDate = from.ToInstant();

            //FromSql($"SELECT AVG(Temperature), AVG(Humidity), AVG(Pressure) FROM Telemetries LEFT JOIN Devices ON Telemetries.Device = Devices.DeviceId
            // WHERE Devices.DeviceName = {deviceName} AND Telemetries.MeasuredTime...")

            var result = await _appDbContext.Telemetries.AsQueryable()
                .Include(x => x.Device)
                .Where(c => c.Device.DeviceName.Equals(deviceName.ToLower()))
                .Where(c =>  c.MeasuredTime >= fromLastDate)
                .Where(c =>  c.MeasuredTime <= toDate)
                .GroupBy(c => c.Device)
                .Select(g => new {Temperature = g.Average(c => c.Temperature), Humidity = g.Average(c => c.Humidity), Pressure = g.Average(c => c.Pressure)})
                .FirstOrDefaultAsync();


            return Ok(result);
        }

        [HttpGet("[action]")]
        [ProducesResponseType(typeof(Dictionary<string, DateTime>), StatusCodes.Status200OK)]
        public IActionResult DeviceActivityTable() {
            Dictionary<string, DateTime> table = _systemStatusService.GetAllLastSeen();
            return Ok(table);
        }


        /////////----
        
        [HttpGet("device/{deviceName}/[action]")]
        public void SendSwitch([FromRoute] string deviceName, [FromQuery] BrokerAccessService.SwitchStates state = BrokerAccessService.SwitchStates.Switch) {
            _brokerAccessService.SendSwitch(deviceName.ToLower(), state);
        }

        [HttpGet("[action]")]
        public void SendGlobalSwitch([FromQuery] BrokerAccessService.SwitchStates state = BrokerAccessService.SwitchStates.Switch) {
            _brokerAccessService.SendGlobalSwitch(state);
        }
        public record struct TelemetryDto(float Temperature, float Humidity, float Pressure, DateTime SubmitedTime, DateTime MeasuredTime);
    }
}
