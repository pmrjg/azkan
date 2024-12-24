use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents, allocator::StandardCommandBufferAllocator};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, physical::PhysicalDeviceType};
use vulkano::image::{ImageAccess, SwapchainImage, view::ImageView};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::{self, AcquireError, Swapchain, SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::{Version, VulkanLibrary};

use vulkano_win::VkSurfaceBuild;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use std::sync::Arc;

fn main() {
    let instance = {
        let library = VulkanLibrary::new().unwrap();
        let extensions = vulkano_win::required_extensions(&library);

        Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: extensions,
                enumerate_portability: true,
                max_api_version: Some(Version::V1_2),
                ..Default::default()
            }
        ).unwrap()
    };

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone()).unwrap();
    
    // physical device
    
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = instance.enumerate_physical_devices().unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.graphics && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| {
            match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5
            }
        }).expect("No suitable physical device found");
    
    // Device (Logical)
    let (device, mut queues) = Device::new(physical_device, 
                                           DeviceCreateInfo {
                                               enabled_extensions: device_extensions,
                                               queue_create_infos: vec![QueueCreateInfo {
                                                   queue_family_index,
                                                   ..Default::default()
                                               }],
                                               ..Default::default()
                                           },).unwrap();
    
    // Queues
    let queue = queues.next().unwrap();
    
    // Swapchain
    let (mut swapchain, images) = {
        let caps = device.physical_device().surface_capabilities(&surface, Default::default()).unwrap();
        
        let usage = caps.supported_usage_flags;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        
        let image_format = Some(
          device.physical_device().surface_formats(&surface, Default::default()).unwrap()[0].0,  
        );
        
        let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
        let image_extent: [u32; 2] = window.inner_size().into();
        
        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count,
                image_format,
                image_extent,
                image_usage: usage,
                composite_alpha: alpha,
                ..Default::default()
            }
        ).unwrap()
    };
}