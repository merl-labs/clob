# Hypertree Library Documentation

## Overview

The Hypertree library is the core innovation enabling Manifest's memory-efficient orderbook. It provides data structures that **do not own their underlying byte arrays**, allowing multiple overlapping data structures to share the same memory region within a single Solana account.

## Core Concepts

### Memory Sharing Architecture
Traditional data structures own their memory, making it impossible to efficiently pack multiple structures together. Hypertree solves this by:

1. **Uniform Node Sizes**: All nodes are exactly the same size (80 bytes for markets, 64 bytes for globals)
2. **Shared Memory Pool**: Multiple trees operate on the same byte array
3. **Index-Based Pointers**: Uses `DataIndex` (u32) instead of memory pointers
4. **Interleaved Storage**: Bids, asks, seats, and free blocks can occupy any position

### DataIndex Pointer System
```rust
pub type DataIndex = u32;
pub const NIL: DataIndex = DataIndex::MAX; // Null pointer equivalent
```

**Key Features:**
- 32-bit indices instead of 64-bit pointers (saves memory)
- Platform-independent (no raw pointer arithmetic)
- Bounds-checkable for safety
- NIL constant represents null/empty

## Core Traits and Interfaces

### Payload Trait
All data stored in hypertree nodes must implement:
```rust
pub trait Payload: Zeroable + Pod + PartialOrd + Ord + PartialEq + Eq + Display {}
```

**Requirements:**
- `Zeroable`: Can be safely zeroed
- `Pod`: Plain Old Data (no references/pointers)
- `Ord`: Total ordering for tree operations
- `Display`: Debug/logging support

### HyperTreeReadOperations
Core read interface for all hypertree structures:
```rust
pub trait HyperTreeReadOperations<'a> {
    fn lookup_index<V: Payload>(&'a self, value: &V) -> DataIndex;
    fn lookup_max_index<V: Payload>(&'a self) -> DataIndex;
    fn get_max_index(&self) -> DataIndex;
    fn get_root_index(&self) -> DataIndex;
    fn get_next_lower_index<V: Payload>(&'a self, index: DataIndex) -> DataIndex;
    fn get_next_higher_index<V: Payload>(&'a self, index: DataIndex) -> DataIndex;
}
```

### HyperTreeWriteOperations
Write interface for modifying trees:
```rust
pub trait HyperTreeWriteOperations<'a, V: Payload> {
    fn insert(&mut self, index: DataIndex, value: V);
    fn remove_by_index(&mut self, index: DataIndex);
}
```

## Red-Black Tree Implementation

### RBNode Structure
The fundamental building block of all trees:
```rust
pub struct RBNode<V> {
    left: DataIndex,           // Left child index
    right: DataIndex,          // Right child index  
    parent: DataIndex,         // Parent index
    color: Color,              // Red or Black
    payload_type: u8,          // Optional type identifier
    _unused_padding: u16,      // Alignment padding
    value: V,                  // Actual payload data
}
```

**Memory Layout:**
- **16 bytes overhead**: Tree structure (left, right, parent, color, etc.)
- **Remaining bytes**: Payload data (64 bytes for markets, 48 bytes for globals)

### Color Enum
```rust
pub enum Color {
    Black = 0,
    Red = 1,
}
```

Red-black tree properties ensure O(log n) operations:
1. Every node is either red or black
2. Root is always black
3. Red nodes have black children
4. All paths from root to leaves have same number of black nodes

### RedBlackTree Structure
```rust
pub struct RedBlackTree<'a, V: Payload> {
    root_index: DataIndex,     // Root node index
    data: &'a mut [u8],        // Shared memory array
    max_index: DataIndex,      // Cached max for O(1) access
    phantom: PhantomData<&'a V>,
}
```

**Key Features:**
- **O(log n)** insert, delete, lookup
- **O(1)** max access (cached)
- **Self-balancing** via red-black properties
- **Memory efficient** via shared storage

### Tree Operations

#### Insertion
```rust
fn insert(&mut self, index: DataIndex, value: V) {
    // 1. Create new red node
    let new_node = RBNode {
        left: NIL,
        right: NIL, 
        parent: NIL,
        color: Color::Red,
        value,
        // ...
    };
    
    // 2. Insert as in BST
    self.insert_node_no_fix(new_node, index);
    
    // 3. Fix red-black violations
    self.insert_fix(index);
}
```

#### Deletion
```rust
fn remove_by_index(&mut self, index: DataIndex) {
    // 1. Standard BST deletion
    // 2. Fix red-black violations if needed
    // 3. Update max cache if necessary
}
```

#### Rotations
Tree balancing via left and right rotations:
```rust
fn rotate_left(&mut self, index: DataIndex) {
    //     G              P
    //   /   \          /   \
    //  U     P   =>   G     X
    //      /   \    /   \
    //     Y     X  U     Y
}

fn rotate_right(&mut self, index: DataIndex) {
    //     G              P  
    //   /   \          /   \
    //  P     U   =>   X     G
    // / \                 / \
    //X   Y               Y   U
}
```

## Memory Management

### Helper Functions
Safe memory access via byte-level operations:
```rust
pub fn get_helper<T: Get>(data: &[u8], index: DataIndex) -> &T {
    let index_usize = index as usize;
    bytemuck::from_bytes(&data[index_usize..index_usize + size_of::<T>()])
}

pub fn get_mut_helper<T: Get>(data: &mut [u8], index: DataIndex) -> &mut T {
    let index_usize = index as usize;
    bytemuck::from_bytes_mut(&mut data[index_usize..index_usize + size_of::<T>()])
}
```

**Safety Features:**
- Bounds checking on array access
- Type-safe byte interpretation via `bytemuck`
- No raw pointer arithmetic
- Compile-time size verification

### Free List Management
Efficient allocation/deallocation without fragmentation:

```rust
pub struct FreeList<'a, T: Pod> {
    head_index: DataIndex,     // Head of free list
    data: &'a mut [u8],        // Shared memory
    phantom: PhantomData<&'a T>,
}

pub struct FreeListNode<T> {
    next_index: DataIndex,     // Next free block
    node_inner: T,             // Unused payload space
}
```

**Operations:**
```rust
// Allocate a free block
fn remove(&mut self) -> DataIndex {
    let free_index = self.head_index;
    let head = get_mut_helper::<FreeListNode<T>>(self.data, free_index);
    self.head_index = head.next_index;
    free_index
}

// Deallocate a block
fn add(&mut self, index: DataIndex) {
    let node = get_mut_helper::<FreeListNode<T>>(self.data, index);
    node.node_inner = T::zeroed();
    node.next_index = self.head_index;
    self.head_index = index;
}
```

## Iterator Support

### HyperTreeValueReadOnlyIterator
Efficient tree traversal:
```rust
pub struct HyperTreeValueReadOnlyIterator<'a, T, V> {
    tree: &'a T,
    index: DataIndex,
    phantom: PhantomData<&'a V>,
}

impl Iterator for HyperTreeValueReadOnlyIterator<'a, T, V> {
    type Item = (DataIndex, &'a V);
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == NIL {
            None
        } else {
            let node = get_helper::<RBNode<V>>(self.tree.data(), self.index);
            let current_index = self.index;
            self.index = self.tree.get_next_lower_index::<V>(self.index);
            Some((current_index, node.get_value()))
        }
    }
}
```

**Usage:**
```rust
// Iterate through all orders in price order
for (index, order) in market.get_bids().iter::<RestingOrder>() {
    println!("Order at {}: {:?}", index, order);
}
```

## Left-Leaning Red-Black Trees (LLRB)

Alternative implementation with simpler balancing:
```rust
pub struct LLRB<'a, V: Payload> {
    root_index: DataIndex,
    data: &'a mut [u8],
    max_index: DataIndex,
    phantom: PhantomData<&'a V>,
}
```

**Differences from standard RB trees:**
- Simpler insertion/deletion logic
- All red links lean left
- Fewer rotation cases
- Slightly different performance characteristics

## Performance Characteristics

### Time Complexity
| Operation | Red-Black Tree | LLRB | Free List |
|-----------|---------------|------|-----------|
| Insert | O(log n) | O(log n) | O(1) |
| Delete | O(log n) | O(log n) | O(1) |
| Lookup | O(log n) | O(log n) | N/A |
| Max | O(1) | O(1) | N/A |
| Iterate | O(n) | O(n) | O(n) |

### Space Complexity
- **Node overhead**: 16 bytes per node
- **Memory sharing**: Multiple trees in same space
- **No fragmentation**: Free list prevents memory holes
- **Cache efficiency**: Uniform node sizes improve locality

## Integration with Manifest

### Market Trees
```rust
// Three trees sharing same memory space
let bids: RedBlackTree<RestingOrder> = RedBlackTree::new(
    dynamic_data, 
    market.bids_root_index, 
    market.bids_best_index
);

let asks: RedBlackTree<RestingOrder> = RedBlackTree::new(
    dynamic_data,
    market.asks_root_index, 
    market.asks_best_index
);

let seats: RedBlackTree<ClaimedSeat> = RedBlackTree::new(
    dynamic_data,
    market.claimed_seats_root_index,
    NIL
);
```

### Memory Layout Example
```
Dynamic Memory (shared by all trees):
┌─────────────────────────────────────────────────────────────────┐
│ [Bid] [Ask] [Seat] [Free] [Bid] [Ask] [Free] [Seat] [Bid] ... │
└─────────────────────────────────────────────────────────────────┘
   ↑     ↑     ↑      ↑      ↑     ↑      ↑      ↑      ↑
   80B   80B   80B    80B    80B   80B    80B    80B    80B
```

## Debugging and Verification

### Tree Validation
```rust
fn verify_rb_tree<V: Payload>(&self) {
    // Check red-black properties:
    // 1. Root is black
    // 2. No red-red parent-child pairs  
    // 3. Equal black height on all paths
    // 4. BST ordering property
}
```

### Debug Utilities
```rust
fn debug_print<V: Payload>(&self);     // Print tree structure
fn depth<V: Payload>(&self, index: DataIndex) -> i32;  // Node depth
fn pretty_print<V: Payload>(&self);    // Visual tree representation
```

## Safety and Formal Verification

The hypertree library has been formally verified to ensure:
- **Red-black tree properties** are maintained
- **Memory safety** with no buffer overflows
- **Correctness** of tree operations
- **Invariant preservation** across all operations

This verification provides mathematical guarantees about the correctness and safety of the hypertree implementation, making it suitable for high-stakes financial applications like Manifest.
