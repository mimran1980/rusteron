use libaeron_driver_sys as aeron_driver;

use std::ffi::CStr;
use std::rc::Rc;
use libaeron_driver_sys::aeron_driver_context_t;
use rusteron_common::{CExt, ManagedCResource};

pub struct AeronContext {
    resource: Rc<ManagedCResource<aeron_driver_context_t>>,
}

impl AeronContext {
    pub fn new() -> rusteron_common::Result<Self, Box<dyn std::error::Error>> {
        let resource = ManagedCResource::new(
            |ctx| unsafe { aeron_driver::aeron_driver_context_init(ctx) },
            |ctx| unsafe { aeron_driver::aeron_driver_context_close(ctx) },
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

    pub fn aeron_driver_context_set_dir(&mut self, value: &str) -> rusteron_common::Result<(), i32> {
        unsafe { aeron_driver::aeron_driver_context_set_dir(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_dir(&self) -> rusteron_common::Result<&str, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_dir(self.resource.get())}.into()
    }
    pub fn aeron_driver_context_set_dir_warn_if_exists(&mut self, value: bool) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_dir_warn_if_exists(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_dir_warn_if_exists(&self) -> bool {
        unsafe { aeron_driver::aeron_driver_context_get_dir_warn_if_exists(self.resource.get())}
    }
    pub fn aeron_driver_context_set_threading_mode(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_threading_mode(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_threading_mode(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_threading_mode(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_dir_delete_on_start(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_dir_delete_on_start(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_dir_delete_on_start(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_dir_delete_on_start(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_dir_delete_on_shutdown(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_dir_delete_on_shutdown(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_dir_delete_on_shutdown(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_dir_delete_on_shutdown(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_to_conductor_buffer_length(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_to_conductor_buffer_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_to_conductor_buffer_length(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_to_conductor_buffer_length(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_to_clients_buffer_length(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_to_clients_buffer_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_to_clients_buffer_length(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_to_clients_buffer_length(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_counters_buffer_length(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_counters_buffer_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_counters_buffer_length(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_counters_buffer_length(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_error_buffer_length(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_error_buffer_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_error_buffer_length(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_error_buffer_length(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_client_liveness_timeout_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_client_liveness_timeout_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_client_liveness_timeout_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_client_liveness_timeout_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_term_buffer_length(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_term_buffer_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_term_buffer_length(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_term_buffer_length(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_ipc_term_buffer_length(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_ipc_term_buffer_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_ipc_term_buffer_length(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_ipc_term_buffer_length(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_term_buffer_sparse_file(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_term_buffer_sparse_file(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_term_buffer_sparse_file(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_term_buffer_sparse_file(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_perform_storage_checks(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_perform_storage_checks(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_perform_storage_checks(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_perform_storage_checks(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_low_file_store_warning_threshold(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_low_file_store_warning_threshold(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_low_file_store_warning_threshold(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_low_file_store_warning_threshold(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_spies_simulate_connection(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_spies_simulate_connection(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_spies_simulate_connection(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_spies_simulate_connection(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_file_page_size(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_file_page_size(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_file_page_size(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_file_page_size(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_mtu_length(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_mtu_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_mtu_length(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_mtu_length(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_ipc_mtu_length(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_ipc_mtu_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_ipc_mtu_length(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_ipc_mtu_length(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_ipc_publication_term_window_length(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_ipc_publication_term_window_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_ipc_publication_term_window_length(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_ipc_publication_term_window_length(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_publication_term_window_length(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_publication_term_window_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_publication_term_window_length(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_publication_term_window_length(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_publication_linger_timeout_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_publication_linger_timeout_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_publication_linger_timeout_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_publication_linger_timeout_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_socket_so_rcvbuf(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_socket_so_rcvbuf(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_socket_so_rcvbuf(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_socket_so_rcvbuf(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_socket_so_sndbuf(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_socket_so_sndbuf(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_socket_so_sndbuf(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_socket_so_sndbuf(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_socket_multicast_ttl(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_socket_multicast_ttl(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_socket_multicast_ttl(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_socket_multicast_ttl(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_send_to_status_poll_ratio(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_send_to_status_poll_ratio(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_send_to_status_poll_ratio(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_send_to_status_poll_ratio(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_rcv_status_message_timeout_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_rcv_status_message_timeout_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_rcv_status_message_timeout_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_rcv_status_message_timeout_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_multicast_flowcontrol_supplier(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_multicast_flowcontrol_supplier(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_multicast_flowcontrol_supplier(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_multicast_flowcontrol_supplier(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_unicast_flowcontrol_supplier(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_unicast_flowcontrol_supplier(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_unicast_flowcontrol_supplier(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_unicast_flowcontrol_supplier(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_image_liveness_timeout_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_image_liveness_timeout_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_image_liveness_timeout_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_image_liveness_timeout_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_rcv_initial_window_length(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_rcv_initial_window_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_rcv_initial_window_length(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_rcv_initial_window_length(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_congestioncontrol_supplier(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_congestioncontrol_supplier(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_congestioncontrol_supplier(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_congestioncontrol_supplier(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_loss_report_buffer_length(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_loss_report_buffer_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_loss_report_buffer_length(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_loss_report_buffer_length(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_publication_unblock_timeout_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_publication_unblock_timeout_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_publication_unblock_timeout_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_publication_unblock_timeout_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_publication_connection_timeout_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_publication_connection_timeout_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_publication_connection_timeout_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_publication_connection_timeout_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_timer_interval_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_timer_interval_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_timer_interval_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_timer_interval_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_sender_idle_strategy(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_sender_idle_strategy(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_sender_idle_strategy(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_sender_idle_strategy(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_conductor_idle_strategy(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_conductor_idle_strategy(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_conductor_idle_strategy(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_conductor_idle_strategy(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_receiver_idle_strategy(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_receiver_idle_strategy(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_receiver_idle_strategy(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_receiver_idle_strategy(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_sharednetwork_idle_strategy(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_sharednetwork_idle_strategy(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_sharednetwork_idle_strategy(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_sharednetwork_idle_strategy(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_shared_idle_strategy(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_shared_idle_strategy(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_shared_idle_strategy(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_shared_idle_strategy(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_sender_idle_strategy_init_args(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_sender_idle_strategy_init_args(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_sender_idle_strategy_init_args(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_sender_idle_strategy_init_args(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_conductor_idle_strategy_init_args(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_conductor_idle_strategy_init_args(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_conductor_idle_strategy_init_args(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_conductor_idle_strategy_init_args(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_receiver_idle_strategy_init_args(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_receiver_idle_strategy_init_args(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_receiver_idle_strategy_init_args(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_receiver_idle_strategy_init_args(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_sharednetwork_idle_strategy_init_args(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_sharednetwork_idle_strategy_init_args(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_sharednetwork_idle_strategy_init_args(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_sharednetwork_idle_strategy_init_args(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_shared_idle_strategy_init_args(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_shared_idle_strategy_init_args(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_shared_idle_strategy_init_args(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_shared_idle_strategy_init_args(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_agent_on_start_function(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_agent_on_start_function(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_agent_on_start_function(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_agent_on_start_function(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_get_agent_on_start_state(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_agent_on_start_state(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_counters_free_to_reuse_timeout_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_counters_free_to_reuse_timeout_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_counters_free_to_reuse_timeout_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_counters_free_to_reuse_timeout_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_flow_control_receiver_timeout_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_flow_control_receiver_timeout_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_flow_control_receiver_timeout_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_flow_control_receiver_timeout_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_flow_control_group_tag(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_flow_control_group_tag(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_flow_control_group_tag(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_flow_control_group_tag(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_flow_control_group_min_size(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_flow_control_group_min_size(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_flow_control_group_min_size(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_flow_control_group_min_size(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_receiver_group_tag(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_receiver_group_tag(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_receiver_group_tag_is_present(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_receiver_group_tag_is_present(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_get_receiver_group_tag_value(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_receiver_group_tag_value(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_driver_termination_validator(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_driver_termination_validator(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_driver_termination_validator(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_driver_termination_validator(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_get_driver_termination_validator_state(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_driver_termination_validator_state(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_driver_termination_hook(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_driver_termination_hook(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_driver_termination_hook(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_driver_termination_hook(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_get_driver_termination_hook_state(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_driver_termination_hook_state(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_print_configuration(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_print_configuration(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_print_configuration(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_print_configuration(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_reliable_stream(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_reliable_stream(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_reliable_stream(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_reliable_stream(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_tether_subscriptions(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_tether_subscriptions(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_tether_subscriptions(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_tether_subscriptions(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_untethered_window_limit_timeout_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_untethered_window_limit_timeout_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_untethered_window_limit_timeout_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_untethered_window_limit_timeout_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_untethered_resting_timeout_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_untethered_resting_timeout_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_untethered_resting_timeout_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_untethered_resting_timeout_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_driver_timeout_ms(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_driver_timeout_ms(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_driver_timeout_ms(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_driver_timeout_ms(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_nak_multicast_group_size(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_nak_multicast_group_size(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_nak_multicast_group_size(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_nak_multicast_group_size(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_nak_multicast_max_backoff_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_nak_multicast_max_backoff_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_nak_multicast_max_backoff_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_nak_multicast_max_backoff_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_nak_unicast_delay_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_nak_unicast_delay_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_nak_unicast_delay_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_nak_unicast_delay_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_nak_unicast_retry_delay_ratio(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_nak_unicast_retry_delay_ratio(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_nak_unicast_retry_delay_ratio(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_nak_unicast_retry_delay_ratio(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_max_resend(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_max_resend(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_max_resend(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_max_resend(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_retransmit_unicast_delay_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_retransmit_unicast_delay_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_retransmit_unicast_delay_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_retransmit_unicast_delay_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_retransmit_unicast_linger_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_retransmit_unicast_linger_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_retransmit_unicast_linger_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_retransmit_unicast_linger_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_receiver_group_consideration(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_receiver_group_consideration(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_receiver_group_consideration(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_receiver_group_consideration(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_rejoin_stream(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_rejoin_stream(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_rejoin_stream(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_rejoin_stream(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_connect_enabled(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_connect_enabled(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_connect_enabled(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_connect_enabled(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_udp_channel_transport_bindings(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_udp_channel_transport_bindings(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_udp_channel_transport_bindings(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_udp_channel_transport_bindings(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_udp_channel_outgoing_interceptors(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_udp_channel_outgoing_interceptors(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_udp_channel_outgoing_interceptors(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_udp_channel_outgoing_interceptors(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_udp_channel_incoming_interceptors(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_udp_channel_incoming_interceptors(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_udp_channel_incoming_interceptors(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_udp_channel_incoming_interceptors(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_publication_reserved_session_id_low(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_publication_reserved_session_id_low(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_publication_reserved_session_id_low(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_publication_reserved_session_id_low(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_publication_reserved_session_id_high(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_publication_reserved_session_id_high(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_publication_reserved_session_id_high(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_publication_reserved_session_id_high(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_resolver_name(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_resolver_name(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_resolver_name(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_resolver_name(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_resolver_interface(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_resolver_interface(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_resolver_interface(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_resolver_interface(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_resolver_bootstrap_neighbor(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_resolver_bootstrap_neighbor(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_resolver_bootstrap_neighbor(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_resolver_bootstrap_neighbor(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_name_resolver_supplier(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_name_resolver_supplier(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_name_resolver_supplier(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_name_resolver_supplier(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_name_resolver_init_args(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_name_resolver_init_args(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_name_resolver_init_args(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_name_resolver_init_args(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_re_resolution_check_interval_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_re_resolution_check_interval_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_re_resolution_check_interval_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_re_resolution_check_interval_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_conductor_duty_cycle_tracker(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_conductor_duty_cycle_tracker(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_conductor_duty_cycle_tracker(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_conductor_duty_cycle_tracker(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_sender_duty_cycle_tracker(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_sender_duty_cycle_tracker(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_sender_duty_cycle_tracker(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_sender_duty_cycle_tracker(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_receiver_duty_cycle_tracker(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_receiver_duty_cycle_tracker(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_receiver_duty_cycle_tracker(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_receiver_duty_cycle_tracker(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_name_resolver_time_tracker(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_name_resolver_time_tracker(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_name_resolver_time_tracker(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_name_resolver_time_tracker(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_sender_wildcard_port_range(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_sender_wildcard_port_range(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_sender_wildcard_port_range(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_sender_wildcard_port_range(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_receiver_wildcard_port_range(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_receiver_wildcard_port_range(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_receiver_wildcard_port_range(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_receiver_wildcard_port_range(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_sender_port_manager(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_sender_port_manager(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_sender_port_manager(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_sender_port_manager(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_receiver_port_manager(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_receiver_port_manager(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_receiver_port_manager(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_receiver_port_manager(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_conductor_cycle_threshold_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_conductor_cycle_threshold_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_conductor_cycle_threshold_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_conductor_cycle_threshold_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_sender_cycle_threshold_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_sender_cycle_threshold_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_sender_cycle_threshold_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_sender_cycle_threshold_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_receiver_cycle_threshold_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_receiver_cycle_threshold_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_receiver_cycle_threshold_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_receiver_cycle_threshold_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_name_resolver_threshold_ns(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_name_resolver_threshold_ns(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_name_resolver_threshold_ns(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_name_resolver_threshold_ns(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_receiver_io_vector_capacity(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_receiver_io_vector_capacity(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_receiver_io_vector_capacity(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_receiver_io_vector_capacity(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_sender_io_vector_capacity(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_sender_io_vector_capacity(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_sender_io_vector_capacity(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_sender_io_vector_capacity(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_network_publication_max_messages_per_send(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_network_publication_max_messages_per_send(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_network_publication_max_messages_per_send(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_network_publication_max_messages_per_send(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_resource_free_limit(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_resource_free_limit(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_resource_free_limit(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_resource_free_limit(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_async_executor_threads(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_async_executor_threads(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_async_executor_threads(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_async_executor_threads(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_enable_experimental_features(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_enable_experimental_features(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_enable_experimental_features(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_enable_experimental_features(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_set_stream_session_limit(&mut self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_set_stream_session_limit(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_get_stream_session_limit(&self) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_get_stream_session_limit(self.resource.get())}.to_result()
    }
    pub fn aeron_driver_context_init(&self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_init(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_close(&self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_close(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_print_configuration(&self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_print_configuration(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_validate_mtu_length(&self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_validate_mtu_length(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_run_storage_checks(&self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_run_storage_checks(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_bindings_clientd_create_entries(&self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_bindings_clientd_create_entries(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_bindings_clientd_delete_entries(&self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_bindings_clientd_delete_entries(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_bindings_clientd_find_first_free_index(&self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_bindings_clientd_find_first_free_index(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_bindings_clientd_find(&self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_bindings_clientd_find(self.resource.get(), value.into())}.to_result()
    }
    pub fn aeron_driver_context_bindings_clientd_get_or_find_first_free_entry(&self, value: _) -> rusteron_common::Result<_, i32> {
        unsafe { aeron_driver::aeron_driver_context_bindings_clientd_get_or_find_first_free_entry(self.resource.get(), value.into())}.to_result()
    }



}

pub struct AeronDriver {
    resource: Rc<ManagedCResource<aeron_driver::aeron_driver_t>>,
}

impl AeronDriver {
    pub fn new(context: &AeronContext) -> rusteron_common::Result<Self, Box<dyn std::error::Error>> {
        let resource = ManagedCResource::new(
            |driver| unsafe { aeron_driver::aeron_driver_init(driver, context.resource.get()) },
            |driver| unsafe { aeron_driver::aeron_driver_close(driver) },
        )
        .map_err(|error_code| {
            format!("failed to initialise aeron driver error code {error_code}")
        })?;

        Ok(Self { resource: Rc::new(resource) })
    }

    pub fn start(&self) -> rusteron_common::Result<(), Box<dyn std::error::Error>> {
        let result = unsafe { aeron_driver::aeron_driver_start(self.resource.get(), false) };
        if result < 0 {
            return Err(format!("failed to start aeron driver error code {result}").into());
        }
        Ok(())
    }

    // Add methods specific to AeronDriver
    pub fn do_work(&self) {
        while unsafe { aeron_driver::aeron_driver_main_do_work(self.resource.get()) } != 0 {
            // busy spin
        }
    }
}

fn threading_mode_to_str(mode: aeron_driver::aeron_threading_mode_t) -> &'static str {
    match mode {
        aeron_driver::aeron_threading_mode_enum::AERON_THREADING_MODE_DEDICATED => "DEDICATED",
        aeron_driver::aeron_threading_mode_enum::AERON_THREADING_MODE_SHARED_NETWORK => {
            "SHARED_NETWORK"
        }
        aeron_driver::aeron_threading_mode_enum::AERON_THREADING_MODE_SHARED => "SHARED",
        aeron_driver::aeron_threading_mode_enum::AERON_THREADING_MODE_INVOKER => "INVOKER",
        _ => "UNKNOWN",
    }
}

fn print_aeron_config(context: *mut aeron_driver::aeron_driver_context_t) -> rusteron_common::Result<()> {
    let config_entries = vec![
        (
            "dir",
            format!("{:?}", unsafe {
                CStr::from_ptr(aeron_driver::aeron_driver_context_get_dir(context))
            }),
        ),
        (
            "dir_warn_if_exists",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_dir_warn_if_exists(context)
            }),
        ),
        (
            "threading_mode",
            threading_mode_to_str(unsafe {
                aeron_driver::aeron_driver_context_get_threading_mode(context)
            })
            .to_string(),
        ),
        (
            "dir_delete_on_start",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_dir_delete_on_start(context)
            }),
        ),
        (
            "dir_delete_on_shutdown",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_dir_delete_on_shutdown(context)
            }),
        ),
        (
            "to_conductor_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_to_conductor_buffer_length(context)
            }),
        ),
        (
            "to_clients_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_to_clients_buffer_length(context)
            }),
        ),
        (
            "counters_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_counters_buffer_length(context)
            }),
        ),
        (
            "error_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_error_buffer_length(context)
            }),
        ),
        (
            "client_liveness_timeout_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_client_liveness_timeout_ns(context)
            }),
        ),
        (
            "term_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_term_buffer_length(context)
            }),
        ),
        (
            "ipc_term_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_ipc_term_buffer_length(context)
            }),
        ),
        (
            "term_buffer_sparse_file",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_term_buffer_sparse_file(context)
            }),
        ),
        (
            "perform_storage_checks",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_perform_storage_checks(context)
            }),
        ),
        (
            "low_file_store_warning_threshold",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_low_file_store_warning_threshold(context)
            }),
        ),
        (
            "spies_simulate_connection",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_spies_simulate_connection(context)
            }),
        ),
        (
            "file_page_size",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_file_page_size(context)
            }),
        ),
        (
            "mtu_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_mtu_length(context)
            }),
        ),
        (
            "ipc_mtu_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_ipc_mtu_length(context)
            }),
        ),
        (
            "ipc_publication_term_window_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_ipc_publication_term_window_length(context)
            }),
        ),
        (
            "publication_term_window_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_publication_term_window_length(context)
            }),
        ),
        (
            "publication_linger_timeout_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_publication_linger_timeout_ns(context)
            }),
        ),
        (
            "socket_so_rcvbuf",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_socket_so_rcvbuf(context)
            }),
        ),
        (
            "socket_so_sndbuf",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_socket_so_sndbuf(context)
            }),
        ),
        (
            "socket_multicast_ttl",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_socket_multicast_ttl(context)
            }),
        ),
        (
            "send_to_status_poll_ratio",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_send_to_status_poll_ratio(context)
            }),
        ),
        (
            "rcv_status_message_timeout_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_rcv_status_message_timeout_ns(context)
            }),
        ),
        (
            "multicast_flowcontrol_supplier",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_multicast_flowcontrol_supplier(context)
            }),
        ),
        (
            "unicast_flowcontrol_supplier",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_unicast_flowcontrol_supplier(context)
            }),
        ),
        (
            "image_liveness_timeout_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_image_liveness_timeout_ns(context)
            }),
        ),
        (
            "rcv_initial_window_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_rcv_initial_window_length(context)
            }),
        ),
        (
            "congestioncontrol_supplier",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_congestioncontrol_supplier(context)
            }),
        ),
        (
            "loss_report_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_loss_report_buffer_length(context)
            }),
        ),
        (
            "publication_unblock_timeout_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_publication_unblock_timeout_ns(context)
            }),
        ),
        (
            "publication_connection_timeout_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_publication_connection_timeout_ns(context)
            }),
        ),
        (
            "timer_interval_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_timer_interval_ns(context)
            }),
        ),
        (
            "sender_idle_strategy",
            format!("{:?}", unsafe {
                CStr::from_ptr(aeron_driver::aeron_driver_context_get_sender_idle_strategy(
                    context,
                ))
            }),
        ),
        (
            "conductor_idle_strategy",
            format!("{:?}", unsafe {
                CStr::from_ptr(
                    aeron_driver::aeron_driver_context_get_conductor_idle_strategy(context),
                )
            }),
        ),
        (
            "receiver_idle_strategy",
            format!("{:?}", unsafe {
                CStr::from_ptr(
                    aeron_driver::aeron_driver_context_get_receiver_idle_strategy(context),
                )
            }),
        ),
        (
            "sharednetwork_idle_strategy",
            format!("{:?}", unsafe {
                CStr::from_ptr(
                    aeron_driver::aeron_driver_context_get_sharednetwork_idle_strategy(context),
                )
            }),
        ),
        (
            "shared_idle_strategy",
            format!("{:?}", unsafe {
                CStr::from_ptr(aeron_driver::aeron_driver_context_get_shared_idle_strategy(
                    context,
                ))
            }),
        ),
        (
            "sender_idle_strategy_init_args",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_sender_idle_strategy_init_args(context)
            }),
        ),
        (
            "conductor_idle_strategy_init_args",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_conductor_idle_strategy_init_args(context)
            }),
        ),
        (
            "receiver_idle_strategy_init_args",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_receiver_idle_strategy_init_args(context)
            }),
        ),
        (
            "sharednetwork_idle_strategy_init_args",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_sharednetwork_idle_strategy_init_args(
                    context,
                )
            }),
        ),
        (
            "shared_idle_strategy_init_args",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_shared_idle_strategy_init_args(context)
            }),
        ),
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
