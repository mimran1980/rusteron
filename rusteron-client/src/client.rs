use libaeron_sys as aeron_driver;
use aeron_driver::*;
use std::ffi::CStr;
use std::rc::Rc;
use rusteron_common::ManagedCResource;

pub struct AeronContext {
    resource: Rc<ManagedCResource<aeron_context_t>>,
}

impl AeronContext {
    pub fn new() -> rusteron_common::Result<Self, Box<dyn std::error::Error>> {
        let resource = ManagedCResource::new(
            |ctx| unsafe { aeron_driver::aeron_context_init(ctx) },
            |ctx| unsafe { aeron_driver::aeron_context_close(ctx) },
        )
            .map_err(|error_code| {
                format!("failed to initialise aeron context error code {error_code}")
            })?;

        Ok(Self { resource : Rc::new(resource) })
    }

    // Add methods specific to AeronContext
    pub fn print_config(&self) -> rusteron_common::Result<(), Box<dyn std::error::Error>> {
        print_aeron_config(self.resource.get())?;
        Ok(())
    }
}

pub struct AeronDriver {
    resource: Rc<ManagedCResource<aeron_driver::aeron_t>>,
}

impl AeronDriver {
    pub fn new(context: &AeronContext) -> rusteron_common::Result<Self, Box<dyn std::error::Error>> {
        let resource = ManagedCResource::new(
            |driver| unsafe { aeron_driver::aeron_init(driver, context.resource.get()) },
            |driver| unsafe { aeron_driver::aeron_close(driver) },
        )
            .map_err(|error_code| {
                format!("failed to initialise aeron driver error code {error_code}")
            })?;

        Ok(Self { resource : Rc::new(resource) })
    }

    pub fn start(&self) -> rusteron_common::Result<(), Box<dyn std::error::Error>> {
        let result = unsafe { aeron_driver::aeron_start(self.resource.get()) };
        if result < 0 {
            return Err(format!("failed to start aeron driver error code {result}").into());
        }
        Ok(())
    }

    // Add methods specific to AeronDriver
    pub fn do_work(&self) {
        while unsafe { aeron_driver::aeron_main_do_work(self.resource.get()) } != 0 {
            // busy spin
        }
    }
}

fn print_aeron_config(context: *mut aeron_driver::aeron_context_t) -> rusteron_common::Result<()> {
    let config_entries = vec![
        (
            "dir",
            format!("{:?}", unsafe {
                CStr::from_ptr(aeron_driver::aeron_context_get_dir(context))
            }),
        ),
        ("aeron_context_get_dir", format!("{:?}", unsafe { aeron_driver::aeron_context_get_dir(context) })),
        ("aeron_context_get_driver_timeout_ms", format!("{:?}", unsafe { aeron_driver::aeron_context_get_driver_timeout_ms(context) })),
        ("aeron_context_get_keepalive_interval_ns", format!("{:?}", unsafe { aeron_driver::aeron_context_get_keepalive_interval_ns(context) })),
        ("aeron_context_get_resource_linger_duration_ns", format!("{:?}", unsafe { aeron_driver::aeron_context_get_resource_linger_duration_ns(context) })),
        ("aeron_context_get_idle_sleep_duration_ns", format!("{:?}", unsafe { aeron_driver::aeron_context_get_idle_sleep_duration_ns(context) })),
        ("aeron_context_get_pre_touch_mapped_memory", format!("{:?}", unsafe { aeron_driver::aeron_context_get_pre_touch_mapped_memory(context) })),
        ("aeron_context_get_client_name", format!("{:?}", unsafe { CStr::from_ptr(aeron_driver::aeron_context_get_client_name(context)) })),
        ("aeron_context_get_error_handler", format!("{:?}", unsafe { aeron_driver::aeron_context_get_error_handler(context) })),
        ("aeron_context_get_error_handler_clientd", format!("{:?}", unsafe { aeron_driver::aeron_context_get_error_handler_clientd(context) })),
        ("aeron_context_get_use_conductor_agent_invoker", format!("{:?}", unsafe { aeron_driver::aeron_context_get_use_conductor_agent_invoker(context) })),
        ("aeron_context_get_agent_on_start_function", format!("{:?}", unsafe { aeron_driver::aeron_context_get_agent_on_start_function(context) })),
        ("aeron_context_get_agent_on_start_state", format!("{:?}", unsafe { aeron_driver::aeron_context_get_agent_on_start_state(context) })),
    ];

    // Find the maximum length of the keys
    let max_key_len = config_entries
        .iter()
        .map(|(key, _)| key.len() + 2)
        .max()
        .unwrap_or(0);

    // Print the aligned configuration entries
    for (key, value) in config_entries {
        println!("{:width$}: {}", key, value, width = max_key_len);
    }

    println!();

    Ok(())
}
