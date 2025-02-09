use sysinfo::{Networks, System};

pub fn print_system_information() {
    // Please note that we use "new_all" to ensure that all lists of
    // CPUs and processes are filled!
    let mut sys = System::new_all();
    // First we update all information of our `System` struct.
    sys.refresh_all();
    println!("=> system:");
    // RAM and swap information:
    println!("total memory: {} bytes", sys.total_memory());
    println!("used memory : {} bytes", sys.used_memory());
    println!("available memory : {} bytes", sys.available_memory());
    println!("total swap  : {} bytes", sys.total_swap());
    println!("used swap   : {} bytes", sys.used_swap());
    // Display system information:
    println!("CPU architecture:        {:?}", System::cpu_arch());
    println!("System name:             {:?}", System::name());
    println!("System kernel version:   {:?}", System::kernel_version());
    println!("System OS version:       {:?}", System::os_version());
    println!("System host name:        {:?}", System::host_name());
    // Number of CPUs:
    println!("Number of CPUs: {}", sys.cpus().len());
    let networks = Networks::new_with_refreshed_list();
    for (_, network) in &networks {
        println!("Internet Protocol (IP) Networks: {:?}", network.ip_networks());
        println!("MAC address: {}", network.mac_address());
        println!("Maximum Transfer Unit (MTU): {}", network.mtu());
    }
}
// TODO: Make work
pub async fn print_gpu_information() {
    let (device, _) = init_wgpu_device().await.unwrap();
    println!("wgpu device: {:?}", device);
}
// TODO: Make work
async fn init_wgpu_device() -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
    // Start an "Instance", which is the context for all things wgpu.
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    // An "Adapter" is a handle to a physical graphics/compute device.
    let adapter = match instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(), // None | LowPower | HighPerformance
            compatible_surface: None,
            force_fallback_adapter: false, // If needed to force CPU fallback?
        })
        .await
    {
        | Some(adapter) => adapter,
        | None => panic!("Failed to request adapter"),
    };
    // Request a connection to a physical device,
    // and also access to the queue for its command buffers.
    // It is possible, if necessary, to add a description of required features.
    adapter.request_device(&Default::default(), None).await
}
