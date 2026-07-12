/*
 * Trace Control support functions
 *
 * This file is part of System Informer.
 */

#ifndef _NTWMI_H
#define _NTWMI_H

#ifndef _TRACEHANDLE_DEFINED
#define _TRACEHANDLE_DEFINED
// Obsolete - prefer PROCESSTRACE_HANDLE or CONTROLTRACE_ID.
typedef ULONG64 TRACEHANDLE, *PTRACEHANDLE;
#endif

// Used to read the events from a trace file or real-time trace session (via
// ProcessTrace). The handle is invalid if it contains the value
// INVALID_PROCESSTRACE_HANDLE. Obtain the handle by calling an OpenTrace
// function (e.g.  OpenTrace, OpenTraceFromFile, OpenTraceFromRealTimeLogger).
// Close the handle by calling CloseTrace.
typedef ULONG64 PROCESSTRACE_HANDLE;

// Used to identify a trace collection session. The id is invalid if it
// contains the value (CONTROLTRACE_ID)0. Obtain the id from StartTrace or from
// the Wnode.HistoricalContext field of the EVENT_TRACE_PROPERTIES returned by
// ControlTrace(0, sessionName, ...). The id is valid until the trace stops and
// does not need to be closed by the user.
typedef ULONG64 CONTROLTRACE_ID;
