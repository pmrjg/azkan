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
use vulkano::format::ClearValue;

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
    
    // Allocators
    let command_buffer_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());
    
    // Shaders
    
    // Renderpass
    let render_pass = vulkano::single_pass_renderpass!(device.clone(), attachments: {
        color: {
            load: Clear,
            store: Store,
            format: swapchain.image_format(),
            samples: 1,
        }
    },
    pass: {
            color: [color],
            depth_stencil: {}
        }).unwrap();
    
    //Graphics Pipeline
    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    };
    
    // FRamebuffers
    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut viewport);
    
    let mut recreate_swapchain = false;
    let mut previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);
    
    
    event_loop.run(move |event, _, control_flow| {
        previous_frame_end
            .as_mut()
            .take()
            .unwrap()
            .cleanup_finished();
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested, ..
            } => {*control_flow = ControlFlow::Exit;},
            Event::WindowEvent {event: WindowEvent::Resized(_), ..} => {recreate_swapchain = true;}, 
            Event::RedrawEventsCleared => {

                if recreate_swapchain {
                    let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();
                    let image_extent: [u32; 2] = window.inner_size().into();

                    let (new_swapchain, new_images) = match swapchain.recreate(SwapchainCreateInfo {
                        image_extent,
                        ..swapchain.create_info()
                    }) {
                        Ok(r) => r,
                        Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                        Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                    };

                    swapchain = new_swapchain;
                    framebuffers =
                        window_size_dependent_setup(&new_images, render_pass.clone(), &mut viewport);
                    recreate_swapchain = false;
                }

                let (image_index, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }
                
                let clear_values:Vec<Option<ClearValue>> = vec![Some([0.0, 0.68, 1.0, 1.0].into())];
                // Command Buffers
                let mut cmd_buffer_builder = AutoCommandBufferBuilder::primary(
                    &command_buffer_allocator,
                    queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                    .unwrap();

                cmd_buffer_builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values,
                            ..RenderPassBeginInfo::framebuffer(
                                framebuffers[image_index as usize].clone(),
                            )
                        },
                        SubpassContents::Inline,
                    )
                    .unwrap()
                    .end_render_pass()
                    .unwrap();

                let command_buffer = cmd_buffer_builder.build().unwrap();

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
                    )
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(Box::new(future) as Box<_>);
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
                    }
                }
            },
            _ => {}
        }
    });
}

fn window_size_dependent_setup(images: &[Arc<SwapchainImage>], render_pass: Arc<RenderPass>, viewport: &mut Viewport) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];
    
    images.iter().map(|image| {
        let view = ImageView::new_default(image.clone()).unwrap();
        
        Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            },).unwrap()
    }).collect::<Vec<_>>()
}