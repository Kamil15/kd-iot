﻿using Google.Protobuf;
using KdIoT.Server.Data;
using KdIoT.Server.Services;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Namotion.Reflection;

var builder = WebApplication.CreateBuilder(args);

// Add services to the container.
// Learn more about configuring Swagger/OpenAPI at https://aka.ms/aspnetcore/swashbuckle
builder.Services.AddEndpointsApiExplorer();
builder.Services.AddControllers();

builder.Services.AddSingleton<BrokerAccessService>();
builder.Services.AddHostedService<BrokerAccessService>(provider => provider.GetRequiredService<BrokerAccessService>());
builder.Services.AddDbContext<AppDbContext>(options => options.UseNpgsql("Host=postgres;Database=kdiotserver_db;Username=kdiotserver;Password=pass5"));

//builder.Services.AddSwaggerGen(c => {
//    c.ResolveConflictingActions(apiDescriptions => apiDescriptions.First());
//});

builder.Services.AddOpenApiDocument();

var app = builder.Build();


using (var scope = app.Services.CreateScope()) {
    var services = scope.ServiceProvider;
    try {
        var context = services.GetRequiredService<AppDbContext>();
        context.Database.Migrate();
    } catch (Exception ex) {
        var logger = services.GetRequiredService<ILogger<Program>>();
        logger.LogError(ex, "An error occurred creating the DB.");
    }
}



app.MapControllers();


//if (app.Environment.IsDevelopment()) { }
app.UseOpenApi();
app.UseSwaggerUi();

var folder = Environment.SpecialFolder.LocalApplicationData;
var path = Environment.GetFolderPath(folder);

Console.WriteLine($"{path}"); // /home/kamil/.local/share

app.Run();
