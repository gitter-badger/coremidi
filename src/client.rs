use core_foundation::string::CFString;
use core_foundation::base::{OSStatus, TCFType};

use coremidi_sys::{
    MIDIClientRef, MIDIClientCreate, MIDIClientDispose, MIDINotification,
    MIDIPortRef, MIDIOutputPortCreate, MIDIEndpointRef, MIDISourceCreate
};

use coremidi_sys_ext::{
    MIDIPacketList, MIDIInputPortCreate, MIDIDestinationCreate
};

use std::ops::Deref;
use std::mem;
use std::ptr;

use Object;
use Client;
use Port;
use OutputPort;
use InputPort;
use Endpoint;
use VirtualSource;
use VirtualDestination;
use PacketList;
use notifications::Notification;

impl Client {
    /// Creates a new CoreMIDI client with support for notifications.
    /// See [MIDIClientCreate](https://developer.apple.com/reference/coremidi/1495360-midiclientcreate).
    ///
    pub fn new_with_notifications<F: Fn(Notification)>(name: &str, callback: F) -> Result<Client, OSStatus> {
        let client_name = CFString::new(name);
        let mut client_ref: MIDIClientRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDIClientCreate(
            client_name.as_concrete_TypeRef(),
            Some(Self::notify_proc::<F> as extern "C" fn(_, _)),
            &callback as *const _ as *mut ::libc::c_void,
            &mut client_ref)
        };
        if status == 0 { Ok(Client { object: Object(client_ref) }) } else { Err(status) }
    }

    /// Creates a new CoreMIDI client.
    /// See [MIDIClientCreate](https://developer.apple.com/reference/coremidi/1495360-midiclientcreate).
    ///
    pub fn new(name: &str) -> Result<Client, OSStatus> {
        let client_name = CFString::new(name);
        let mut client_ref: MIDIClientRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDIClientCreate(
            client_name.as_concrete_TypeRef(),
            None, ptr::null_mut(),
            &mut client_ref)
        };
        if status == 0 { Ok(Client { object: Object(client_ref) }) } else { Err(status) }
    }

    /// Creates an output port through which the client may send outgoing MIDI messages to any MIDI destination.
    /// See [MIDIOutputPortCreate](https://developer.apple.com/reference/coremidi/1495166-midioutputportcreate).
    ///
    pub fn output_port(&self, name: &str) -> Result<OutputPort, OSStatus> {
        let port_name = CFString::new(name);
        let mut port_ref: MIDIPortRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDIOutputPortCreate(
            self.object.0,
            port_name.as_concrete_TypeRef(),
            &mut port_ref)
        };
        if status == 0 { Ok(OutputPort { port: Port { object: Object(port_ref) } }) } else { Err(status) }
    }

    /// Creates an input port through which the client may receive incoming MIDI messages from any MIDI source.
    /// See [MIDIInputPortCreate](https://developer.apple.com/reference/coremidi/1495225-midiinputportcreate).
    ///
    pub fn input_port<F>(&self, name: &str, callback: F) -> Result<InputPort, OSStatus>
            where F: Fn(PacketList) {

        let port_name = CFString::new(name);
        let mut port_ref: MIDIPortRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDIInputPortCreate(
            self.object.0,
            port_name.as_concrete_TypeRef(),
            Some(Self::read_proc::<F> as extern "C" fn(_, _, _)),
            &callback as *const _ as *mut ::libc::c_void,
            &mut port_ref)
        };
        if status == 0 { Ok(InputPort { port: Port { object: Object(port_ref) } }) } else { Err(status) }
    }

    /// Creates a virtual source in the client.
    /// See [MIDISourceCreate](https://developer.apple.com/reference/coremidi/1495212-midisourcecreate).
    ///
    pub fn virtual_source(&self, name: &str) -> Result<VirtualSource, OSStatus> {
        let virtual_source_name = CFString::new(name);
        let mut virtual_source: MIDIEndpointRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDISourceCreate(
            self.object.0,
            virtual_source_name.as_concrete_TypeRef(),
            &mut virtual_source)
        };
        if status == 0 { Ok(VirtualSource { endpoint: Endpoint { object: Object(virtual_source) } }) } else { Err(status) }
    }

    /// Creates a virtual destination in the client.
    /// See [MIDIDestinationCreate](https://developer.apple.com/reference/coremidi/1495347-mididestinationcreate).
    ///
    pub fn virtual_destination<F>(&self, name: &str, callback: F) -> Result<VirtualDestination, OSStatus>
            where F: Fn(PacketList) {

        let virtual_destination_name = CFString::new(name);
        let mut virtual_destination: MIDIEndpointRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDIDestinationCreate(
            self.object.0,
            virtual_destination_name.as_concrete_TypeRef(),
            Some(Self::read_proc::<F> as extern "C" fn(_, _, _)),
            &callback as *const _ as *mut ::libc::c_void,
            &mut virtual_destination)
        };
        if status == 0 { Ok(VirtualDestination { endpoint: Endpoint { object: Object(virtual_destination) } }) } else { Err(status) }
    }

    extern "C" fn notify_proc<F: Fn(Notification)>(
            notification_ptr: *const MIDINotification,
            ref_con: *mut ::libc::c_void) {

        let _ = ::std::panic::catch_unwind(|| unsafe {
            match Notification::from(&*notification_ptr) {
                Ok(notification) => {
                    let ref callback = *(ref_con as *const F);
                    callback(notification);
                },
                Err(_) => {} // Skip unknown notifications
            }
        });
    }

    extern "C" fn read_proc<F: Fn(PacketList)>(
            pktlist: *const MIDIPacketList,
            read_proc_ref_con: *mut ::libc::c_void,
            _: *mut ::libc::c_void) { //srcConnRefCon

        let _ = ::std::panic::catch_unwind(|| unsafe {
            let packet_list = PacketList(pktlist);
            let ref callback = *(read_proc_ref_con as *const F);
            callback(packet_list);
        });
    }
}

impl Deref for Client {
    type Target = Object;

    fn deref(&self) -> &Object {
        &self.object
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe { MIDIClientDispose(self.object.0) };
    }
}
