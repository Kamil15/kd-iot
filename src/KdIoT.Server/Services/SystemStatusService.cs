using System.Collections.Concurrent;

namespace KdIoT.Server.Services {
    public class SystemStatusService {
        private readonly SemaphoreSlim _semaphore;
        public ConcurrentDictionary<string, DeviceActivityValue> DeviceAcitvity {get; private set;}

        public SystemStatusService() {
            _semaphore = new SemaphoreSlim(0, 1);
            DeviceAcitvity = new();
        }


        public void UpdateLastSeen(string device, DateTime lastSeen) {
            DeviceAcitvity.AddOrUpdate(device, (key) => new (lastSeen), (key, oldvalue) => new (lastSeen));
        }

        public DateTime GetLastSeen(string device) {
            return DeviceAcitvity.GetValueOrDefault(device).LastSeen;
        }

        public Dictionary<string, DateTime> GetAllLastSeen() {
            Dictionary<string, DateTime> returnList = new();
            foreach(var item in DeviceAcitvity) {
                returnList.Add(item.Key, item.Value.LastSeen);
            }

            return returnList;

        }

    }



    public record struct DeviceActivityValue(DateTime LastSeen);
}