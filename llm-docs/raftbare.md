# Raftbare Crate Documentation

## Crate Summary

### Overview

`raftbare` is a minimal but feature-complete, I/O-free implementation of the Raft distributed consensus algorithm. The crate provides a pure state machine implementation of Raft, where all I/O operations are represented as actions that users must execute themselves.

### Key Use Cases and Benefits

- **Distributed Consensus**: Implement strongly consistent replicated state machines across multiple nodes
- **Custom I/O Integration**: Full control over I/O execution allows integration with any async runtime, threading model, or storage backend
- **Educational**: Straightforward implementation closely following the Raft paper
- **Testing**: I/O-free design enables deterministic testing and simulation
- **Flexibility**: Users control timeouts, storage mechanisms, and network communication

### Design Philosophy

The crate focuses exclusively on the core Raft algorithm logic. It:
- **Separates concerns**: Algorithm logic is decoupled from I/O operations
- **Generates actions**: Instead of performing I/O directly, the crate emits `Action` values that describe what I/O operations are needed
- **Maintains simplicity**: Straightforward implementation without unnecessary abstractions
- **Delegates details**: Convenience features not described in the Raft paper are left to users

---

## API Reference

### Core Types

#### `raftbare::Node`

The main struct representing a Raft node in a cluster.

##### Constructor Methods

```rust
pub fn raftbare::Node::start(id: raftbare::NodeId) -> raftbare::Node
```

Starts a new node that is not yet part of any cluster. To create a cluster, call `raftbare::Node::create_cluster()` after starting.

**Parameters:**
- `id`: Unique identifier for this node

**Returns:** A new `raftbare::Node` instance in follower role with no configuration

---

```rust
pub fn raftbare::Node::restart(
    id: raftbare::NodeId,
    current_term: raftbare::Term,
    voted_for: Option<raftbare::NodeId>,
    log: raftbare::Log
) -> raftbare::Node
```

Restarts a node with previously persisted state. Used after a node crash or restart.

**Parameters:**
- `id`: Node identifier
- `current_term`: Restored current term from persistent storage
- `voted_for`: Restored voted-for value from persistent storage
- `log`: Restored log from persistent storage

**Returns:** A restarted `raftbare::Node` instance with actions queued

**Note:** The Raft algorithm assumes reliable persistent storage. Corrupted or partially lost logs may violate safety guarantees depending on the application.

---

##### Cluster Management Methods

```rust
pub fn raftbare::Node::create_cluster(
    &mut self,
    initial_voters: &[raftbare::NodeId]
) -> raftbare::LogPosition
```

Creates a new cluster with the specified initial voting members.

**Parameters:**
- `initial_voters`: Array of node IDs that will be voters in the new cluster (must contain at least one node)

**Returns:** 
- `raftbare::LogPosition` of the cluster configuration entry if successful
- `raftbare::LogPosition::INVALID` if preconditions are not met

**Preconditions:**
- Node must be newly started (log position is `ZERO`)
- Node must not already be in a cluster
- `initial_voters` must not be empty

**Side Effects:** Queues actions that must be executed to complete cluster creation

---

```rust
pub fn raftbare::Node::propose_config(
    &mut self,
    new_config: raftbare::ClusterConfig
) -> raftbare::LogPosition
```

Proposes a new cluster configuration, potentially entering joint consensus.

**Parameters:**
- `new_config`: The new cluster configuration to propose

**Returns:**
- `raftbare::LogPosition` of the configuration entry if successful
- `raftbare::LogPosition::INVALID` if preconditions are not met

**Preconditions:**
- Node must be the leader
- `new_config.voters` must equal current configuration's voters
- Voters and non-voters must be disjoint sets
- No other configuration change can be in progress

**Note:** If `new_config.new_voters` is non-empty, the cluster enters joint consensus mode. Once committed, a final configuration is automatically proposed.

---

##### Command Proposal Methods

```rust
pub fn raftbare::Node::propose_command(&mut self) -> raftbare::LogPosition
```

Proposes a user-defined command for replication.

**Returns:**
- `raftbare::LogPosition` identifying the command's log entry if successful
- `raftbare::LogPosition::INVALID` if node is not the leader

**Preconditions:**
- Node must be the leader

**Note:** The crate does not manage command data. Users must maintain a mapping from log positions to actual command data.

---

##### Message Handling Methods

```rust
pub fn raftbare::Node::handle_message(&mut self, msg: raftbare::Message)
```

Processes an incoming Raft RPC message from another node.

**Parameters:**
- `msg`: The message to process (RequestVote or AppendEntries RPC)

**Side Effects:** May queue actions and transition node role/state

---

```rust
pub fn raftbare::Node::handle_election_timeout(&mut self)
```

Handles an election timeout event.

**Description:** Should be called when the timeout set by `raftbare::Action::SetElectionTimeout` expires. Behavior depends on current role:
- **Follower**: Transitions to candidate and starts election
- **Candidate**: Starts new election with incremented term
- **Leader**: Sends heartbeat to followers

---

```rust
pub fn raftbare::Node::handle_snapshot_installed(
    &mut self,
    last_included_position: raftbare::LogPosition,
    last_included_config: raftbare::ClusterConfig
) -> bool
```

Updates the node's log to reflect a snapshot installation.

**Parameters:**
- `last_included_position`: Position of the last entry included in the snapshot
- `last_included_config`: Cluster configuration at `last_included_position`

**Returns:** `true` if snapshot was successfully installed, `false` if preconditions failed

**Preconditions:**
- `last_included_position` must be valid (contained in log or beyond commit index for non-leaders)
- `last_included_config` must match the configuration at `last_included_position`

**Effect:** Truncates log entries up to `last_included_position`

---

##### State Query Methods

```rust
pub fn raftbare::Node::id(&self) -> raftbare::NodeId
```

Returns the identifier of this node.

---

```rust
pub fn raftbare::Node::role(&self) -> raftbare::Role
```

Returns the current role of this node (Leader, Candidate, or Follower).

---

```rust
pub fn raftbare::Node::current_term(&self) -> raftbare::Term
```

Returns the current term number.

---

```rust
pub fn raftbare::Node::voted_for(&self) -> Option<raftbare::NodeId>
```

Returns the node ID this node voted for in the current term, if any.

---

```rust
pub fn raftbare::Node::log(&self) -> &raftbare::Log
```

Returns a reference to the in-memory log representation.

---

```rust
pub fn raftbare::Node::commit_index(&self) -> raftbare::LogIndex
```

Returns the highest log index known to be committed.

---

```rust
pub fn raftbare::Node::config(&self) -> &raftbare::ClusterConfig
```

Returns the current cluster configuration. Equivalent to `self.log().latest_config()`.

---

```rust
pub fn raftbare::Node::peers(&self) -> impl Iterator<Item = raftbare::NodeId>
```

Returns an iterator over peer node IDs (all cluster members except this node).

---

```rust
pub fn raftbare::Node::actions(&self) -> &raftbare::Actions
```

Returns a reference to pending actions that need execution.

---

```rust
pub fn raftbare::Node::actions_mut(&mut self) -> &mut raftbare::Actions
```

Returns a mutable reference to pending actions. Users should consume and execute these actions.

---

```rust
pub fn raftbare::Node::get_commit_status(
    &self,
    position: raftbare::LogPosition
) -> raftbare::CommitStatus
```

Returns the commit status of the log entry at the given position.

**Parameters:**
- `position`: Log position to query

**Returns:** One of:
- `raftbare::CommitStatus::InProgress`: Entry is being committed
- `raftbare::CommitStatus::Committed`: Entry has been committed
- `raftbare::CommitStatus::Rejected`: Entry was rejected (overwritten)
- `raftbare::CommitStatus::Unknown`: Entry removed by snapshot

---

```rust
pub fn raftbare::Node::heartbeat(&mut self) -> bool
```

Sends empty AppendEntries messages to all followers.

**Returns:** `true` if node is leader, `false` otherwise

**Use Case:** Useful for read linearizability - wait for majority acknowledgment before serving reads.

---

### Action Types

#### `raftbare::Action`

Enum representing I/O operations that users must execute.

**Variants:**

```rust
raftbare::Action::SetElectionTimeout
```

Set a new election timeout. The previous timeout is cancelled. When expired, call `raftbare::Node::handle_election_timeout()`.

**Timeout Guidelines:**
- **Leader**: Shorter than follower timeouts (for heartbeats)
- **Candidate**: Random value to avoid election conflicts
- **Follower**: Longer than leader's heartbeat interval

---

```rust
raftbare::Action::SaveCurrentTerm
```

Save `raftbare::Node::current_term()` to persistent storage synchronously before proceeding.

---

```rust
raftbare::Action::SaveVotedFor
```

Save `raftbare::Node::voted_for()` to persistent storage synchronously before proceeding.

---

```rust
raftbare::Action::BroadcastMessage(raftbare::Message)
```

Broadcast the message to all peers (`raftbare::Node::peers()`). Can be executed asynchronously.

---

```rust
raftbare::Action::AppendLogEntries(raftbare::LogEntries)
```

Append log entries to persistent storage. May overwrite existing entries. Should be executed synchronously for correctness (though async is common in practice).

---

```rust
raftbare::Action::SendMessage(raftbare::NodeId, raftbare::Message)
```

Send a message to a specific node. Can be executed asynchronously. Large messages may be truncated using `raftbare::LogEntries::truncate()`.

---

```rust
raftbare::Action::InstallSnapshot(raftbare::NodeId)
```

Install a snapshot on the specified node. Implementation details are user-managed. After completion, call `raftbare::Node::handle_snapshot_installed()`.

---

#### `raftbare::Actions`

A collection of pending actions with prioritized fields.

**Fields (in priority order):**

```rust
pub set_election_timeout: bool
pub save_current_term: bool
pub save_voted_for: bool
pub broadcast_message: Option<raftbare::Message>
pub append_log_entries: Option<raftbare::LogEntries>
pub send_messages: std::collections::BTreeMap<raftbare::NodeId, raftbare::Message>
pub install_snapshots: std::collections::BTreeSet<raftbare::NodeId>
```

**Methods:**

```rust
pub fn raftbare::Actions::is_empty(&self) -> bool
```

Returns `true` if no actions are pending.

---

```rust
pub fn raftbare::Actions::next(&mut self) -> Option<raftbare::Action>
```

Returns the highest priority pending action and removes it. Implements `Iterator` trait.

---

### Message Types

#### `raftbare::Message`

Enum representing Raft RPC messages.

**Variants:**

```rust
raftbare::Message::RequestVoteCall {
    header: raftbare::MessageHeader,
    last_position: raftbare::LogPosition,
}
```

RequestVote RPC call from a candidate.

---

```rust
raftbare::Message::RequestVoteReply {
    header: raftbare::MessageHeader,
    vote_granted: bool,
}
```

RequestVote RPC reply.

---

```rust
raftbare::Message::AppendEntriesCall {
    header: raftbare::MessageHeader,
    commit_index: raftbare::LogIndex,
    entries: raftbare::LogEntries,
}
```

AppendEntries RPC call from the leader.

---

```rust
raftbare::Message::AppendEntriesReply {
    header: raftbare::MessageHeader,
    last_position: raftbare::LogPosition,
}
```

AppendEntries RPC reply. Contains last log position instead of boolean success for faster convergence.

---

**Methods:**

```rust
pub fn raftbare::Message::from(&self) -> raftbare::NodeId
```

Returns the sender's node ID.

---

```rust
pub fn raftbare::Message::term(&self) -> raftbare::Term
```

Returns the sender's term.

---

```rust
pub fn raftbare::Message::seqno(&self) -> raftbare::MessageSeqNo
```

Returns the message sequence number.

---

#### `raftbare::MessageHeader`

Common header for all messages.

**Fields:**

```rust
pub from: raftbare::NodeId
pub term: raftbare::Term
pub seqno: raftbare::MessageSeqNo
```

---

#### `raftbare::MessageSeqNo`

Message sequence number for detecting duplicates and reordering.

**Methods:**

```rust
pub const fn raftbare::MessageSeqNo::new(seqno: u64) -> raftbare::MessageSeqNo
```

Creates a new sequence number.

---

```rust
pub const fn raftbare::MessageSeqNo::get(self) -> u64
```

Returns the sequence number value.

---

### Log Types

#### `raftbare::Log`

In-memory representation of a node's replicated log.

**Constructor:**

```rust
pub const fn raftbare::Log::new(
    snapshot_config: raftbare::ClusterConfig,
    entries: raftbare::LogEntries
) -> raftbare::Log
```

Creates a log with a snapshot configuration and entries.

**Parameters:**
- `snapshot_config`: Configuration at the snapshot point
- `entries`: Log entries after the snapshot

---

**Methods:**

```rust
pub fn raftbare::Log::entries(&self) -> &raftbare::LogEntries
```

Returns a reference to log entries.

---

```rust
pub fn raftbare::Log::last_position(&self) -> raftbare::LogPosition
```

Returns the position of the last entry.

---

```rust
pub fn raftbare::Log::snapshot_position(&self) -> raftbare::LogPosition
```

Returns the position where the snapshot was taken.

---

```rust
pub fn raftbare::Log::snapshot_config(&self) -> &raftbare::ClusterConfig
```

Returns the cluster configuration at the snapshot point.

---

```rust
pub fn raftbare::Log::latest_config(&self) -> &raftbare::ClusterConfig
```

Returns the most recent cluster configuration in the log.

---

```rust
pub fn raftbare::Log::get_position_and_config(
    &self,
    index: raftbare::LogIndex
) -> Option<(raftbare::LogPosition, &raftbare::ClusterConfig)>
```

Returns the position and configuration at the given index. Useful for snapshotting.

**Parameters:**
- `index`: Log index to query

**Returns:** `Some((position, config))` if index is valid, `None` otherwise

---

#### `raftbare::LogEntries`

Compact representation of log entries requiring `O(|terms|) + O(|configs|)` memory.

**Constructors:**

```rust
pub const fn raftbare::LogEntries::new(prev_position: raftbare::LogPosition) -> raftbare::LogEntries
```

Creates an empty log entries collection starting at `prev_position`.

---

```rust
pub fn raftbare::LogEntries::from_iter<I>(
    prev_position: raftbare::LogPosition,
    entries: I
) -> raftbare::LogEntries
where
    I: IntoIterator<Item = raftbare::LogEntry>
```

Creates log entries from an iterator.

**Parameters:**
- `prev_position`: Position immediately before the first entry
- `entries`: Iterator of log entries

---

**Methods:**

```rust
pub fn raftbare::LogEntries::len(&self) -> usize
```

Returns the number of entries.

---

```rust
pub fn raftbare::LogEntries::is_empty(&self) -> bool
```

Returns `true` if there are no entries.

---

```rust
pub fn raftbare::LogEntries::prev_position(&self) -> raftbare::LogPosition
```

Returns the position immediately before the first entry.

---

```rust
pub fn raftbare::LogEntries::last_position(&self) -> raftbare::LogPosition
```

Returns the position of the last entry.

---

```rust
pub fn raftbare::LogEntries::iter(&self) -> impl Iterator<Item = raftbare::LogEntry>
```

Returns an iterator over entries.

---

```rust
pub fn raftbare::LogEntries::iter_with_positions(
    &self
) -> impl Iterator<Item = (raftbare::LogPosition, raftbare::LogEntry)>
```

Returns an iterator over entries with their positions.

---

```rust
pub fn raftbare::LogEntries::contains(&self, position: raftbare::LogPosition) -> bool
```

Returns `true` if the position exists in these entries with matching term.

---

```rust
pub fn raftbare::LogEntries::contains_index(&self, index: raftbare::LogIndex) -> bool
```

Returns `true` if the index is within range (ignores term).

---

```rust
pub fn raftbare::LogEntries::get_term(&self, index: raftbare::LogIndex) -> Option<raftbare::Term>
```

Returns the term at the given index.

---

```rust
pub fn raftbare::LogEntries::get_entry(&self, index: raftbare::LogIndex) -> Option<raftbare::LogEntry>
```

Returns the entry at the given index (excluding the prev_position).

---

```rust
pub fn raftbare::LogEntries::push(&mut self, entry: raftbare::LogEntry)
```

Appends an entry to the end.

---

```rust
pub fn raftbare::LogEntries::truncate(&mut self, len: usize)
```

Keeps the first `len` entries, dropping the rest.

---

#### `raftbare::LogEntry`

Enum representing a single log entry.

**Variants:**

```rust
raftbare::LogEntry::Term(raftbare::Term)
```

Marks the start of a new term with a new leader.

---

```rust
raftbare::LogEntry::ClusterConfig(raftbare::ClusterConfig)
```

Contains a cluster configuration change.

---

```rust
raftbare::LogEntry::Command
```

Represents a user-defined command. The crate does not manage command content; users must maintain the mapping from positions to actual command data.

---

#### `raftbare::LogPosition`

Uniquely identifies a log entry by term and index.

**Constants:**

```rust
pub const raftbare::LogPosition::ZERO: raftbare::LogPosition
```

Initial log position (term 0, index 0).

---

```rust
pub const raftbare::LogPosition::INVALID: raftbare::LogPosition
```

Invalid log position used to signal errors.

---

**Fields:**

```rust
pub term: raftbare::Term
pub index: raftbare::LogIndex
```

**Methods:**

```rust
pub const fn raftbare::LogPosition::is_invalid(self) -> bool
```

Returns `true` if this is the invalid position.

---

#### `raftbare::LogIndex`

Log index (0-based, where 0 is a sentinel value).

**Constants:**

```rust
pub const raftbare::LogIndex::ZERO: raftbare::LogIndex
```

Initial log index (sentinel).

---

**Methods:**

```rust
pub const fn raftbare::LogIndex::new(i: u64) -> raftbare::LogIndex
```

Creates a log index.

---

```rust
pub const fn raftbare::LogIndex::get(self) -> u64
```

Returns the index value.

---

Implements arithmetic operators: `Add`, `AddAssign`, `Sub`, `SubAssign`, and conversion from/to `u64`.

---

#### `raftbare::CommitStatus`

Enum representing the commit status of a log entry.

**Variants:**

```rust
raftbare::CommitStatus::InProgress
```

Entry is currently being committed.

---

```rust
raftbare::CommitStatus::Committed
```

Entry has been successfully committed.

---

```rust
raftbare::CommitStatus::Rejected
```

Entry was rejected (overwritten).

---

```rust
raftbare::CommitStatus::Unknown
```

Entry no longer exists (removed by snapshot).

---

**Methods:**

```rust
pub const fn raftbare::CommitStatus::is_in_progress(self) -> bool
pub const fn raftbare::CommitStatus::is_committed(self) -> bool
pub const fn raftbare::CommitStatus::is_rejected(self) -> bool
pub const fn raftbare::CommitStatus::is_unknown(self) -> bool
```

Predicate methods for checking status.

---

### Configuration Types

#### `raftbare::ClusterConfig`

Cluster membership configuration.

**Fields:**

```rust
pub voters: std::collections::BTreeSet<raftbare::NodeId>
```

Current voting members.

---

```rust
pub new_voters: std::collections::BTreeSet<raftbare::NodeId>
```

New voting members during joint consensus (empty if no configuration change in progress).

---

```rust
pub non_voters: std::collections::BTreeSet<raftbare::NodeId>
```

Non-voting members that replicate log but don't participate in quorum decisions.

---

**Methods:**

```rust
pub fn raftbare::ClusterConfig::new() -> raftbare::ClusterConfig
```

Creates an empty configuration.

---

```rust
pub fn raftbare::ClusterConfig::contains(&self, id: raftbare::NodeId) -> bool
```

Returns `true` if the node is in this configuration.

---

```rust
pub fn raftbare::ClusterConfig::is_joint_consensus(&self) -> bool
```

Returns `true` if in joint consensus (configuration change in progress).

---

```rust
pub fn raftbare::ClusterConfig::unique_nodes(&self) -> impl Iterator<Item = raftbare::NodeId>
```

Returns an iterator over all unique node IDs in sorted order.

---

```rust
pub fn raftbare::ClusterConfig::to_joint_consensus(
    &self,
    adding_voters: &[raftbare::NodeId],
    removing_voters: &[raftbare::NodeId]
) -> raftbare::ClusterConfig
```

Creates a joint consensus configuration by modifying voters.

**Parameters:**
- `adding_voters`: Node IDs to add as voters
- `removing_voters`: Node IDs to remove from voters

**Returns:** New configuration in joint consensus mode

---

### Identity Types

#### `raftbare::NodeId`

Node identifier (wraps `u64`).

**Methods:**

```rust
pub const fn raftbare::NodeId::new(id: u64) -> raftbare::NodeId
```

Creates a node ID.

---

```rust
pub const fn raftbare::NodeId::get(self) -> u64
```

Returns the ID value.

---

Implements arithmetic operators and conversion from/to `u64`.

---

#### `raftbare::Term`

Raft term number.

**Constants:**

```rust
pub const raftbare::Term::ZERO: raftbare::Term
```

Initial term (0).

---

**Methods:**

```rust
pub const fn raftbare::Term::new(t: u64) -> raftbare::Term
```

Creates a term.

---

```rust
pub const fn raftbare::Term::get(self) -> u64
```

Returns the term value.

---

Implements arithmetic operators and conversion from/to `u64`.

---

### Role Type

#### `raftbare::Role`

Enum representing a node's role in the cluster.

**Variants:**

```rust
raftbare::Role::Follower
raftbare::Role::Candidate
raftbare::Role::Leader
```

**Methods:**

```rust
pub const fn raftbare::Role::is_leader(self) -> bool
pub const fn raftbare::Role::is_follower(self) -> bool
pub const fn raftbare::Role::is_candidate(self) -> bool
```

Predicate methods for checking role.

---

## Examples and Common Patterns

### Basic Cluster Creation

```rust
fn create_three_node_cluster() {
    let node_ids = [
        raftbare::NodeId::new(0),
        raftbare::NodeId::new(1),
        raftbare::NodeId::new(2),
    ];

    // Start nodes
    let mut node0 = raftbare::Node::start(node_ids[0]);
    let mut node1 = raftbare::Node::start(node_ids[1]);
    let mut node2 = raftbare::Node::start(node_ids[2]);

    // Create cluster from node0
    let position = node0.create_cluster(&node_ids);
    assert!(!position.is_invalid());

    // Execute actions until cluster is created
    while node0.get_commit_status(position).is_in_progress() {
        for action in node0.actions_mut() {
            match action {
                raftbare::Action::SetElectionTimeout => { /* set timeout */ }
                raftbare::Action::SaveCurrentTerm => { /* persist term */ }
                raftbare::Action::SaveVotedFor => { /* persist vote */ }
                raftbare::Action::BroadcastMessage(msg) => {
                    // Send to node1 and node2
                    node1.handle_message(msg.clone());
                    node2.handle_message(msg);
                }
                raftbare::Action::AppendLogEntries(_) => { /* write to disk */ }
                raftbare::Action::SendMessage(id, msg) => {
                    // Route message to specific node
                }
                raftbare::Action::InstallSnapshot(_) => { /* install snapshot */ }
            }
        }

        // Handle timeouts and forward messages between nodes
        // ...
    }
}
```

### Proposing Commands

```rust
fn propose_and_wait_for_commit(leader: &mut raftbare::Node) -> raftbare::LogPosition {
    let position = leader.propose_command();
    if position.is_invalid() {
        // Not the leader - redirect to voted_for node
        return position;
    }

    // Execute actions
    for action in leader.actions_mut() {
        match action {
            raftbare::Action::AppendLogEntries(entries) => {
                // Map log indices to actual command data before persisting
                for (pos, entry) in entries.iter_with_positions() {
                    if matches!(entry, raftbare::LogEntry::Command) {
                        // Associate pos.index with actual command data
                    }
                }
                // Persist entries...
            }
            raftbare::Action::BroadcastMessage(_) => { /* broadcast */ }
            raftbare::Action::SetElectionTimeout => { /* reset timeout */ }
            _ => { /* handle other actions */ }
        }
    }

    // Wait for commit
    while leader.get_commit_status(position).is_in_progress() {
        // Handle messages, timeouts, etc.
    }

    position
}
```

### Handling Message Delivery

```rust
fn message_delivery_loop(nodes: &mut [raftbare::Node], messages: &[raftbare::Message]) {
    for msg in messages {
        let destination = msg.from(); // In practice, extract actual destination
        
        for node in nodes.iter_mut() {
            if node.id() == destination {
                node.handle_message(msg.clone());
                
                // Process resulting actions
                for action in node.actions_mut() {
                    // Execute actions...
                }
                break;
            }
        }
    }
}
```

### Taking Snapshots

```rust
fn take_snapshot(node: &mut raftbare::Node) {
    let commit_index = node.commit_index();
    
    // Get the configuration at commit index
    let (position, config) = node.log()
        .get_position_and_config(commit_index)
        .expect("commit index must be valid");
    
    // Serialize application state up to commit_index
    // ... user-specific snapshot logic ...
    
    // Update node's log
    let success = node.handle_snapshot_installed(position, config.clone());
    assert!(success);
    
    // Node's log is now truncated
    assert_eq!(node.log().snapshot_position(), position);
}
```

### Configuration Changes

```rust
fn add_node_to_cluster(leader: &mut raftbare::Node, new_node_id: raftbare::NodeId) {
    // Create joint consensus configuration
    let new_config = leader.config().to_joint_consensus(&[new_node_id], &[]);
    
    let position = leader.propose_config(new_config);
    if position.is_invalid() {
        // Check preconditions failed
        return;
    }
    
    // Execute actions and wait for commit
    // Once committed, a second configuration is automatically proposed
    // to finalize the transition
}

fn remove_node_from_cluster(leader: &mut raftbare::Node, old_node_id: raftbare::NodeId) {
    let new_config = leader.config().to_joint_consensus(&[], &[old_node_id]);
    let position = leader.propose_config(new_config);
    // ... wait for commit ...
}
```

### Node Restart Pattern

```rust
fn restart_node_after_crash(
    node_id: raftbare::NodeId,
    storage: &dyn PersistentStorage
) -> raftbare::Node {
    // Load from persistent storage
    let current_term = storage.load_term();
    let voted_for = storage.load_voted_for();
    let log = storage.load_log();
    
    // Restart node
    let mut node = raftbare::Node::restart(node_id, current_term, voted_for, log);
    
    // Execute queued actions
    for action in node.actions_mut() {
        match action {
            raftbare::Action::SetElectionTimeout => { /* set timeout */ }
            _ => { /* other actions shouldn't be present */ }
        }
    }
    
    node
}
```

### Election Timeout Handling

```rust
fn handle_timeout_expiration(node: &mut raftbare::Node) {
    node.handle_election_timeout();
    
    for action in node.actions_mut() {
        match action {
            raftbare::Action::SetElectionTimeout => {
                // Set new timeout based on role:
                let timeout_ms = match node.role() {
                    raftbare::Role::Leader => 50,        // Heartbeat interval
                    raftbare::Role::Candidate => rand_range(150, 300),
                    raftbare::Role::Follower => 300,     // Election timeout
                };
                // Schedule timeout...
            }
            raftbare::Action::SaveCurrentTerm => { /* candidate incremented term */ }
            raftbare::Action::BroadcastMessage(_) => { /* election or heartbeat */ }
            _ => { /* handle other actions */ }
        }
    }
}
```

### Pipelining Commands

```rust
fn pipeline_commands(leader: &mut raftbare::Node, commands: &[Command]) -> Vec<raftbare::LogPosition> {
    let mut positions = Vec::new();
    
    // Propose all commands without executing actions
    for _ in commands {
        let pos = leader.propose_command();
        positions.push(pos);
    }
    
    // Actions are automatically merged/consolidated
    // Execute them all at once
    for action in leader.actions_mut() {
        // Single broadcast contains all entries
        // Single storage write contains all entries
        // etc.
    }
    
    positions
}
```

### Linearizable Reads

```rust
fn linearizable_read(leader: &mut raftbare::Node) -> bool {
    // Send heartbeat and record sequence number
    if !leader.heartbeat() {
        return false; // Not the leader
    }
    
    let mut seqno = raftbare::MessageSeqNo::new(0);
    for action in leader.actions_mut() {
        if let raftbare::Action::BroadcastMessage(msg) = action {
            seqno = msg.seqno();
            // Send message...
        }
    }
    
    // Wait for majority of responses with seqno >= recorded seqno
    // Then perform read from state machine
    true
}
```

### Error Handling Patterns

```rust
fn handle_invalid_operations(node: &mut raftbare::Node) {
    // Check if operations succeed
    let position = node.propose_command();
    if position.is_invalid() {
        // Redirect client to current leader hint
        if let Some(maybe_leader) = node.voted_for() {
            // Try maybe_leader
        }
        return;
    }
    
    // Check configuration change preconditions
    let new_config = node.config().to_joint_consensus(&[raftbare::NodeId::new(3)], &[]);
    let position = node.propose_config(new_config);
    if position.is_invalid() {
        // Configuration change not allowed:
        // - Not the leader, or
        // - Another config change in progress, or
        // - Invalid configuration
    }
}
```

### Checking Commit Status

```rust
fn wait_for_commit_with_timeout(
    node: &mut raftbare::Node,
    position: raftbare::LogPosition,
    max_iterations: usize
) -> raftbare::CommitStatus {
    for _ in 0..max_iterations {
        let status = node.get_commit_status(position);
        
        match status {
            raftbare::CommitStatus::Committed => {
                // Apply to state machine
                return status;
            }
            raftbare::CommitStatus::Rejected => {
                // Entry was overwritten - retry with new leader
                return status;
            }
            raftbare::CommitStatus::Unknown => {
                // Entry removed by snapshot - unknown result
                return status;
            }
            raftbare::CommitStatus::InProgress => {
                // Continue waiting, execute actions
            }
        }
        
        // Execute actions, handle messages, etc.
    }
    
    raftbare::CommitStatus::InProgress // Timeout
}
```
