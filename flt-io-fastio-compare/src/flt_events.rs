//! FileIo / FltIo event definitions for the FLT comparison.
//!
//! Only the minifilter (FltIo*) event classes are needed here, because the
//! `PERF_FLT_*` group masks (0x80100000 / 0x80200000) only enable the
//! FltIoInit (96/97), FltIoCompletion (98/99) and FltIoFailure (100/101)
//! events at version 3.

use fileio_events_macro::fileio_events;

fileio_events! {
    pub enum FltEvent {
        PreOpInitV3 = (96, 3) {
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
        },
        PostOpInitV3 = (97, 3) {
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
        },

        PreOpCompletionV3 = (98, 3) {
            InitialTime: u64 "InitialTime",
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
        },
        PostOpCompletionV3 = (99, 3) {
            InitialTime: u64 "InitialTime",
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
        },

        PreOpFailureV3 = (100, 3) {
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
            Status: u32 "Status",
        },
        PostOpFailureV3 = (101, 3) {
            RoutineAddr: pointer "RoutineAddr",
            FileObject: pointer "FileObject",
            FileContext: pointer "FileContext",
            IrpPtr: pointer "IrpPtr",
            CallbackDataPtr: pointer "CallbackDataPtr",
            MajorFunction: u32 "MajorFunction",
            Status: u32 "Status",
        },
    }
}

/// Compute a stable payload signature (FNV-1a 64-bit) over the parsed field
/// values of a FltIo event, together with the opcode. Two events observed by
/// two different sessions are considered "the same underlying event" when they
/// share the same opcode, version and this signature.
pub fn payload_sig(ev: &FltEvent) -> u64 {
    let mut h = Fnv64::default();
    match ev {
        FltEvent::PreOpInitV3(v) => {
            h.u64(v.RoutineAddr as u64);
            h.u64(v.FileObject as u64);
            h.u64(v.FileContext as u64);
            h.u64(v.IrpPtr as u64);
            h.u64(v.CallbackDataPtr as u64);
            h.u32(v.MajorFunction);
        }
        FltEvent::PostOpInitV3(v) => {
            h.u64(v.RoutineAddr as u64);
            h.u64(v.FileObject as u64);
            h.u64(v.FileContext as u64);
            h.u64(v.IrpPtr as u64);
            h.u64(v.CallbackDataPtr as u64);
            h.u32(v.MajorFunction);
        }
        FltEvent::PreOpCompletionV3(v) => {
            h.u64(v.InitialTime);
            h.u64(v.RoutineAddr as u64);
            h.u64(v.FileObject as u64);
            h.u64(v.FileContext as u64);
            h.u64(v.IrpPtr as u64);
            h.u64(v.CallbackDataPtr as u64);
            h.u32(v.MajorFunction);
        }
        FltEvent::PostOpCompletionV3(v) => {
            h.u64(v.InitialTime);
            h.u64(v.RoutineAddr as u64);
            h.u64(v.FileObject as u64);
            h.u64(v.FileContext as u64);
            h.u64(v.IrpPtr as u64);
            h.u64(v.CallbackDataPtr as u64);
            h.u32(v.MajorFunction);
        }
        FltEvent::PreOpFailureV3(v) => {
            h.u64(v.RoutineAddr as u64);
            h.u64(v.FileObject as u64);
            h.u64(v.FileContext as u64);
            h.u64(v.IrpPtr as u64);
            h.u64(v.CallbackDataPtr as u64);
            h.u32(v.MajorFunction);
            h.u32(v.Status);
        }
        FltEvent::PostOpFailureV3(v) => {
            h.u64(v.RoutineAddr as u64);
            h.u64(v.FileObject as u64);
            h.u64(v.FileContext as u64);
            h.u64(v.IrpPtr as u64);
            h.u64(v.CallbackDataPtr as u64);
            h.u32(v.MajorFunction);
            h.u32(v.Status);
        }
    }
    h.finish()
}

/// Return the MajorFunction value for an event (used for reporting), if any.
pub fn major_function(ev: &FltEvent) -> u32 {
    match ev {
        FltEvent::PreOpInitV3(v) => v.MajorFunction,
        FltEvent::PostOpInitV3(v) => v.MajorFunction,
        FltEvent::PreOpCompletionV3(v) => v.MajorFunction,
        FltEvent::PostOpCompletionV3(v) => v.MajorFunction,
        FltEvent::PreOpFailureV3(v) => v.MajorFunction,
        FltEvent::PostOpFailureV3(v) => v.MajorFunction,
    }
}

/// Tiny FNV-1a (64-bit) hasher.
#[derive(Default)]
struct Fnv64 {
    hash: u64,
}

impl Fnv64 {
    fn byte(&mut self, b: u8) {
        self.hash ^= b as u64;
        self.hash = self.hash.wrapping_mul(0x100000001b3);
    }
    fn bytes(&mut self, b: &[u8]) {
        for &x in b {
            self.byte(x);
        }
    }
    fn u32(&mut self, v: u32) {
        self.bytes(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.bytes(&v.to_le_bytes());
    }
    fn finish(&self) -> u64 {
        self.hash
    }
}