use log::info;
use std::{ffi::c_void, ptr::NonNull};

use objc2::{rc::Retained, runtime::AnyObject, MainThreadMarker};
use objc2_core_audio::{
    kAudioAggregateDeviceIsPrivateKey, kAudioAggregateDeviceNameKey,
    kAudioAggregateDeviceTapAutoStartKey, kAudioAggregateDeviceTapListKey,
    kAudioAggregateDeviceUIDKey, kAudioSubTapDriftCompensationKey, kAudioSubTapUIDKey,
    AudioDeviceCreateIOProcID, AudioDeviceDestroyIOProcID, AudioDeviceIOProcID, AudioDeviceStart,
    AudioDeviceStop, AudioHardwareCreateAggregateDevice, AudioHardwareCreateProcessTap,
    AudioHardwareDestroyProcessTap, AudioObjectID, CATapDescription, CATapMuteBehavior,
};
use objc2_core_audio_types::{AudioBufferList, AudioTimeStamp};
use objc2_core_foundation::CFDictionary;
use objc2_foundation::{ns_string, NSArray, NSDictionary, NSNumber, NSString};

fn cstr_to_nsstring(cstr: &std::ffi::CStr) -> Retained<NSString> {
    NSString::from_str(cstr.to_str().unwrap())
}

pub struct VisualiserAudioTap {
    tap_id: AudioObjectID,
    aggregate_device_id: AudioObjectID,
    tap_io_proc_id: AudioDeviceIOProcID,
    peak_ptr: *mut f32,
}

impl VisualiserAudioTap {
    pub fn setup() -> Self {
        info!("Setting up macOS CoreAudio tap");
        let peak_ptr = Box::into_raw(Box::new(0.0_f32));

        let mtm = MainThreadMarker::new().unwrap();

        let tap_description = unsafe {
            CATapDescription::initStereoGlobalTapButExcludeProcesses(mtm.alloc(), &NSArray::new())
        };
        unsafe {
            tap_description.setMuteBehavior(CATapMuteBehavior::Unmuted);
            tap_description.setName(ns_string!("Joe's Core Audio Tap"));
            tap_description.setPrivate(true);
            tap_description.setExclusive(true);
        }

        let mut tap_id: AudioObjectID = 0;

        unsafe {
            AudioHardwareCreateProcessTap(Some(&tap_description), &raw mut tap_id);
        }

        let tap_uid = unsafe { tap_description.UUID() }.UUIDString();

        let taps = NSArray::from_slice(&[&*NSDictionary::<NSString, AnyObject>::from_slices(
            &[
                &*cstr_to_nsstring(kAudioSubTapUIDKey),
                &*cstr_to_nsstring(kAudioSubTapDriftCompensationKey),
            ],
            &[&*tap_uid, &NSNumber::new_bool(true)],
        )]);

        let aggregate_device_properties = NSDictionary::<NSString, AnyObject>::from_slices(
            &[
                &*cstr_to_nsstring(kAudioAggregateDeviceNameKey),
                &*cstr_to_nsstring(kAudioAggregateDeviceUIDKey),
                &*cstr_to_nsstring(kAudioAggregateDeviceTapListKey),
                &*cstr_to_nsstring(kAudioAggregateDeviceTapAutoStartKey),
                &*cstr_to_nsstring(kAudioAggregateDeviceIsPrivateKey),
            ],
            &[
                ns_string!("JoeCoreAudioTapDevice"),
                ns_string!("com.joebentley.JoeCoreAudioTapDevice"),
                &*taps,
                &NSNumber::new_bool(false),
                &NSNumber::new_bool(true),
            ],
        );

        let mut aggregate_device_id: AudioObjectID = 0;

        unsafe {
            AudioHardwareCreateAggregateDevice(
                AsRef::<CFDictionary<NSString, AnyObject>>::as_ref(&*aggregate_device_properties)
                    .as_opaque(),
                NonNull::new_unchecked(&raw mut aggregate_device_id),
            );
        }

        let mut tap_io_proc_id: AudioDeviceIOProcID = None;
        unsafe {
            AudioDeviceCreateIOProcID(
                aggregate_device_id,
                Some(ioproc_callback),
                peak_ptr as *mut c_void,
                NonNull::new_unchecked(&raw mut tap_io_proc_id),
            );

            AudioDeviceStart(aggregate_device_id, tap_io_proc_id);
        }

        Self {
            tap_id,
            aggregate_device_id,
            tap_io_proc_id,
            peak_ptr,
        }
    }

    /// Get the current peak audio from the pointer. Only call once per frame
    pub fn audio_peak(&self) -> f32 {
        unsafe { *self.peak_ptr }
    }
}

impl Drop for VisualiserAudioTap {
    fn drop(&mut self) {
        info!("Tearing down macOS CoreAudio tap");
        unsafe {
            AudioDeviceStop(self.aggregate_device_id, self.tap_io_proc_id);
            AudioDeviceDestroyIOProcID(self.aggregate_device_id, self.tap_io_proc_id);
            AudioHardwareDestroyProcessTap(self.tap_id);
        }
    }
}

unsafe extern "C-unwind" fn ioproc_callback(
    _in_device: AudioObjectID,
    _in_now: NonNull<AudioTimeStamp>,
    in_input_data: NonNull<AudioBufferList>,
    _in_input_time: NonNull<AudioTimeStamp>,
    _out_output_data: NonNull<AudioBufferList>,
    _in_output_time: NonNull<AudioTimeStamp>,
    in_client_data: *mut c_void,
) -> i32 {
    let peak_ptr = in_client_data as *mut f32;

    let in_input_data_ref = unsafe { in_input_data.as_ref() };
    let num_buffers = in_input_data_ref.mNumberBuffers;

    let mut volume_total = 0.0_f32;
    for buffer in 0..num_buffers {
        let buffer = in_input_data_ref.mBuffers[buffer as usize];
        let num_channels = buffer.mNumberChannels as usize;
        let num_frames = buffer.mDataByteSize as usize / std::mem::size_of::<f32>();
        let num_frames_per_channel = num_frames / num_channels;

        let data = buffer.mData as *mut f32;

        for channel in 0..num_channels {
            let mut volume_accum = 0.0;
            for frame in 0..num_frames_per_channel {
                volume_accum += unsafe { *data.add(frame * num_channels + channel) }.abs();
            }

            volume_accum /= num_frames_per_channel as f32;
            volume_total += volume_accum;
        }
    }

    unsafe {
        peak_ptr.write(volume_total);
    }

    0
}
