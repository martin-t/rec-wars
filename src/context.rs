//! Contexts for running gamelogic.
//!
//! These structs borrow the parts of a client or server's data
//! needed for running gamelogic and other systems
//! on the client, server or a subset on both.
//!
//! The client and server contexts automatically dereference to the shared subset in FrameCtx.
//!
//! ## Reasoning
//!
//! - More code can be shared so the client can use it for client-side prediction.
//! - You can write gamecode without having to worry about where a method is implemented.
//!
//! ## Safety
//!
//! The structs are repr(C) and the shared fields are first and in the same order, so they have the same layout.
//! All fields are references so they have the same alignment too.

use std::{
    mem,
    ops::{Deref, DerefMut},
};

use crate::prelude::*;

/// Context for running gamelogic common to client and server.
#[repr(C)]
pub struct FrameCtx<'a> {
    pub cvars: &'a Cvars,
    pub map: &'a Map,
    pub gs: &'a mut GameState,
}

impl<'a> FrameCtx<'a> {
    pub fn new(cvars: &'a Cvars, map: &'a Map, gs: &'a mut GameState) -> FrameCtx<'a> {
        FrameCtx { cvars, map, gs }
    }
}

/// Context for running client-side gamelogic, rendering and potentially other bookkeeping.
#[repr(C)]
pub struct ClientFrameCtx<'a> {
    pub cvars: &'a Cvars,
    pub map: &'a Map,
    pub gs: &'a mut GameState,
    pub cg: &'a mut ClientGame,
}

impl<'a> ClientFrameCtx<'a> {
    pub fn new(
        cvars: &'a Cvars,
        map: &'a Map,
        gs: &'a mut GameState,
        cg: &'a mut ClientGame,
    ) -> ClientFrameCtx<'a> {
        ClientFrameCtx { cvars, map, gs, cg }
    }
}

/// Context for running server-side gamelogic and potentially other bookkeeping.
#[repr(C)]
pub struct ServerFrameCtx<'a> {
    pub cvars: &'a Cvars,
    pub map: &'a Map,
    pub gs: &'a mut GameState,
    pub sg: &'a mut ServerGame,
}

impl<'a> ServerFrameCtx<'a> {
    pub fn new(
        cvars: &'a Cvars,
        map: &'a Map,
        gs: &'a mut GameState,
        sg: &'a mut ServerGame,
    ) -> ServerFrameCtx<'a> {
        ServerFrameCtx { cvars, map, gs, sg }
    }
}

impl<'a> Deref for ClientFrameCtx<'a> {
    type Target = FrameCtx<'a>;

    #[allow(trivial_casts)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const ClientFrameCtx as *const FrameCtx) }
    }
}

impl DerefMut for ClientFrameCtx<'_> {
    #[allow(trivial_casts)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut ClientFrameCtx as *mut FrameCtx) }
    }
}

impl<'a> Deref for ServerFrameCtx<'a> {
    type Target = FrameCtx<'a>;

    #[allow(trivial_casts)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const ServerFrameCtx as *const FrameCtx) }
    }
}

impl DerefMut for ServerFrameCtx<'_> {
    #[allow(trivial_casts)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut ServerFrameCtx as *mut FrameCtx) }
    }
}

// LATER When offset_of is stabilized (https://github.com/rust-lang/rust/issues/106655),
// use static assert to verify the fields have the same offsets.

static_assert!(mem::align_of::<ClientFrameCtx>() == mem::align_of::<FrameCtx>());
static_assert!(mem::align_of::<ServerFrameCtx>() == mem::align_of::<FrameCtx>());
