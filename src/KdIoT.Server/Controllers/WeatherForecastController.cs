using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;

namespace KdIoT.Server.Controllers {

    [ApiController]
    [Route("[controller]")]
    public class WeatherForecastController : ControllerBase {

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
