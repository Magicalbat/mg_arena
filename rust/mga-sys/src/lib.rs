use std::{ffi::c_void, mem::ManuallyDrop};

#[link(name = "mg_arena")]
extern "C" {
    /// Creates a new arena, returning a pointer to it.
    ///
    /// # Example
    ///
    /// ```
    /// let arena = mga_create(&MGADesc::default() as *const MGADesc);
    ///
    /// // Do stuff
    ///
    /// mga_destroy(arena);
    /// ```
    pub fn mga_create(desc: *const MGADesc) -> *mut MGArena;

    /// Destroys an arena and frees its memory.
    ///
    /// See [`mga_create`] for example.
    pub fn mga_destroy(arena: *mut MGArena);

    /// Allocates `size` bytes in the arena, returning a pointer to the beginning of the allocated memory.
    ///
    /// # Example
    ///
    /// ```
    /// let arena = mga_create(&MGADesc::default() as *const MGADesc);
    ///
    /// let data = mga_push(arena, 1) as *mut u8;
    ///
    /// unsafe {
    ///     *data = 3;
    ///     assert_eq!(*data, 3);
    /// }
    ///
    /// mga_destroy(arena);
    /// ```
    pub fn mga_push(arena: *mut MGArena, size: u64) -> *mut c_void;

    /// Same as [mga_push], but it zeroes out the allocated memory first.
    ///
    /// # Example
    ///
    /// ```
    /// let arena = mga_create(&MGADesc::default() as *const MGADesc);
    ///
    /// let data = mga_push_zero(arena, 1) as *mut u8;
    ///
    /// unsafe {
    ///     assert_eq!(*data, 0);
    /// }
    ///
    /// mga_destroy(arena);
    /// ```
    pub fn mga_push_zero(arena: *mut MGArena, size: u64) -> *mut c_void;

    /// Frees `size` bytes in the arena.
    pub fn mga_pop(arena: *mut MGArena, size: u64);
    /// Frees all bytes after `pos`.
    pub fn mga_pop_to(arena: *mut MGArena, pos: u64);

    /// Begins a temporary arena with the given arena.
    pub fn mga_temp_begin(arena: *mut MGArena) -> MGTempArena;
    /// Ends a temporary arena with the given arena.
    pub fn mga_temp_end(temp: MGTempArena);
}

/// An arena that you can allocate data on, see [`mga_create`].
#[repr(C)]
pub struct MGArena {
    pub pos: u64,

    pub size: u64,
    pub block_size: u64,
    pub align: u32,

    pub _backend: MGArenaBackend,
}

/// An arena descriptor, used to pass information for building the arena. This struct implements [`Default`], which you can use to fill in default arguments.
///
/// # Example
///
/// ```
/// let desc = MGADesc {
///     max_size: mga_mib(4),
///     ..Default::default()
/// };
/// ```
#[repr(C)]
#[derive(Debug, Clone)]
pub struct MGADesc {
    /// Maximum size of the arena, must be set or else the arena will be unable to allocate anything.
    pub max_size: u64,

    /// The size of each page in the arena. Default is platform specific, but set to 4096 if on an unknown platofmr.
    pub page_size: u32,

    /// The amount of pages per block in the arena, defaults to `min(max_size / page_size, 8)`.
    pub pages_per_block: u32,

    /// The alignment, defaults to the size of a pointer `sizeof(void*)`.
    pub align: u32,
}

// Technically could be derived, but I'd rather be explicit
impl Default for MGADesc {
    fn default() -> Self {
        MGADesc {
            max_size: 0,
            page_size: 0,
            pages_per_block: 0,
            align: 0,
        }
    }
}

/// A union that represents different backend kinds.
///
/// # Implementation Note
///
/// I added [`ManuallyDrop`] because it fixed a random error I was getting. I should go back later and try to find a better solution that I actually understand.
#[repr(C)]
pub union MGArenaBackend {
    pub _malloc_arena: ManuallyDrop<MGAMallocArena>,
    pub _reserve_arena: ManuallyDrop<MGAReserveArena>,
}

#[repr(C)]
#[derive(Debug)]
pub struct MGAMallocArena {
    first: *mut MGAMallocNode,
    last: *mut MGAMallocNode,
    num_nodes: u32,
}

/// Used by [`MGAMallocArena`].
#[repr(C)]
#[derive(Debug)]
pub struct MGAMallocNode {
    pub next: *mut MGAMallocNode,
    pub pos: u64,
    pub data: *mut u8,
}

#[repr(C)]
#[derive(Debug)]
pub struct MGAReserveArena {
    commit_pos: u64,
}

/// A temporary arena, see [`mga_temp_begin`].
#[repr(C)]
#[derive(Debug)]
pub struct MGTempArena {
    arena: *mut MGArena,
    pos: u64,
}

/// Returns number of bytes for given KB (1,000 bytes).
pub const fn mga_kb(x: u64) -> u64 {
    x * 1_000
}

/// Returns number of bytes for given MB (1,000,000 bytes).
pub const fn mga_mb(x: u64) -> u64 {
    x * 1_000_000
}

/// Returns number of bytes for given GB (1,000,000,000 bytes).
pub const fn mga_gb(x: u64) -> u64 {
    x * 1_000_000_000
}

/// Returns number of bytes for given KiB (1,024 bytes).
pub const fn mga_kib(x: u64) -> u64 {
    x << 10
}

/// Returns number of bytes for given MiB (1,048,576 bytes).
pub const fn mga_mib(x: u64) -> u64 {
    x << 20
}

/// Returns number of bytes for given MiB (1,073,741,824 bytes).
pub const fn mga_gib(x: u64) -> u64 {
    x << 30
}
