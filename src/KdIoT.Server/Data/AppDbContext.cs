using System;
using System.ComponentModel.DataAnnotations.Schema;
using KdIoT.Server;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.ValueGeneration;
using NodaTime;

namespace KdIoT.Server.Data {
    public class AppDbContext : DbContext {
        public DbSet<Telemetry> Telemetries { get; set; }
        public DbSet<Device> Devices { get; set; }
        //protected override void OnConfiguring(DbContextOptionsBuilder optionsBuilder)
        //    => optionsBuilder.UseNpgsql("Host=my_host;Database=my_db;Username=my_user;Password=my_pw");

        public AppDbContext(DbContextOptions<AppDbContext> dbContextOptions) : base(dbContextOptions) {
            
        }

        protected override void OnModelCreating(ModelBuilder modelBuilder) {
            modelBuilder
                .Entity<Device>()
                .Property(b => b.DeviceId)
                .HasValueGenerator<GuidValueGenerator>();
            
        }
    }

    public class Telemetry {

        public Int64 TelemetryId {get; set;}
        public Device Device {get; set;} = null!;

        public float Temperature {get; set;}
        public float Humidity {get; set;}
        public float Pressure {get; set;}

        public Instant SubmitedTime {get; set;}
        public Instant MeasuredTime {get; set;}
    }

    public class Device {
        public Guid DeviceId {get; set;}
        public string DeviceName {get;set;} = null!;

        public ICollection<Telemetry> Telemetries {get; set;} = new List<Telemetry>();
    }

    /// <summary>
    /// ///////////
    /// </summary>

    public class Blog {
        public int BlogId { get; set; }
        public string Url { get; set; } = null!;

        public List<Post> Posts { get; set; } = null!;
    }

    public class Post {
        public int PostId { get; set; }
        public string Title { get; set; } = null!;
        public string Content { get; set; }  = null!;

        public int BlogId { get; set; }
        public Blog Blog { get; set; } = null!;
    }
}