use coremidi_sys::{
    MIDIGetNumberOfDestinations, MIDIGetDestination,
    MIDIEndpointDispose, ItemCount
};

use Destination;
use properties;

impl Destination {
    /// Create a destination endpoint from its index.
    /// See [MIDIGetDestination](https://developer.apple.com/reference/coremidi/1495108-midigetdestination)
    ///
    pub fn from_index(index: usize) -> Destination {
        let endpoint_ref = unsafe { MIDIGetDestination(index as ItemCount) };
        Destination(endpoint_ref)
    }

    /// Get the display name for the destination endpoint.
    ///
    pub fn display_name(&self) -> Option<String> {
        properties::get_display_name(self.0)
    }
}

impl Drop for Destination {
    fn drop(&mut self) {
        unsafe { MIDIEndpointDispose(self.0) };
    }
}


/// Destination endpoints available in the system.
///
/// The number of destinations available in the system can be retrieved with:
///
/// ```
/// let number_of_destinations = coremidi::Destinations::count();
/// ```
///
/// The destinations in the system can be iterated as:
///
/// ```
/// for destination in coremidi::Destinations {
///   println!("{}", destination.get_display_name());
/// }
/// ```
///
pub struct Destinations;

impl Destinations {
    /// Get the number of destinations available for sending MIDI messages.
    /// See [MIDIGetNumberOfDestinations](https://developer.apple.com/reference/coremidi/1495309-midigetnumberofdestinations).
    ///
    pub fn count() -> usize {
        unsafe { MIDIGetNumberOfDestinations() as usize }
    }
}

impl IntoIterator for Destinations {
    type Item = Destination;
    type IntoIter = DestinationsIterator;

    fn into_iter(self) -> Self::IntoIter {
        DestinationsIterator { index: 0, count: Self::count() }
    }
}

pub struct DestinationsIterator {
    index: usize,
    count: usize
}

impl Iterator for DestinationsIterator {
    type Item = Destination;

    fn next(&mut self) -> Option<Destination> {
        if self.index < self.count {
            let destination = Some(Destination::from_index(self.index));
            self.index += 1;
            destination
        }
        else {
            None
        }
    }
}