// Copyright (c) 2024 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

package iceoryx2

/*
#include "iox2/iceoryx2.h"
#include <stdlib.h>
*/
import "C"
import (
	"context"
	"fmt"
	"time"
)

// Node is the central entry point of iceoryx2. It represents a node in the
// iceoryx2 system. One process can have arbitrary many nodes but usually
// there should be only one node per process.
type Node struct {
	handle      C.iox2_node_h
	serviceType ServiceType
}

// NodeBuilder is used to create a new Node with custom settings.
type NodeBuilder struct {
	handle             C.iox2_node_builder_h
	name               *NodeName
	signalHandlingMode *SignalHandlingMode
	err                error // stores any error encountered during building
}

// NewNodeBuilder creates a new NodeBuilder for constructing a Node.
func NewNodeBuilder() *NodeBuilder {
	handle := C.iox2_node_builder_new(nil)
	return &NodeBuilder{handle: handle}
}

// Name sets the name for the Node being built.
// The name does not have to be unique.
// If the name is invalid, the error is stored and returned by Create().
func (b *NodeBuilder) Name(name string) *NodeBuilder {
	if b.err != nil {
		return b // Don't overwrite existing error
	}
	nodeName, err := NewNodeName(name)
	if err != nil {
		b.err = err
		return b
	}
	b.name = nodeName
	return b
}

// SignalHandlingMode sets the signal handling mode for the Node.
func (b *NodeBuilder) SignalHandlingMode(mode SignalHandlingMode) *NodeBuilder {
	if b.err != nil {
		return b
	}
	b.signalHandlingMode = &mode
	return b
}

// Create creates a new Node with the specified ServiceType.
// Returns any error encountered during the build process or node creation.
func (b *NodeBuilder) Create(serviceType ServiceType) (*Node, error) {
	if b.err != nil {
		return nil, b.err
	}
	if b.handle == nil {
		return nil, ErrNodeBuilderConsumed
	}

	// Set the name if provided
	if b.name != nil {
		ptr := C.iox2_cast_node_name_ptr(b.name.handle)
		C.iox2_node_builder_set_name(&b.handle, ptr)
	}

	var nodeHandle C.iox2_node_h
	result := C.iox2_node_builder_create(b.handle, nil, uint32(serviceType), &nodeHandle)

	// The builder handle is consumed by create
	b.handle = nil

	// Clean up the name
	if b.name != nil {
		b.name.Close()
		b.name = nil
	}

	if result != C.IOX2_OK {
		return nil, NodeCreationError(result)
	}

	return &Node{
		handle:      nodeHandle,
		serviceType: serviceType,
	}, nil
}

// Close releases the resources associated with the Node.
// After calling Close, the Node should not be used.
// Implements io.Closer.
func (n *Node) Close() error {
	if n.handle != nil {
		C.iox2_node_drop(n.handle)
		n.handle = nil
	}
	return nil
}

// Name returns the name of the Node.
func (n *Node) Name() string {
	if n.handle == nil {
		return ""
	}
	ptr := C.iox2_node_name(&n.handle)
	if ptr == nil {
		return ""
	}
	var nameLen C.c_size_t
	cStr := C.iox2_node_name_as_chars(ptr, &nameLen)
	return C.GoStringN(cStr, C.int(nameLen))
}

// Wait waits for the specified duration.
// This is typically used in the main loop to control the cycle time.
// Returns nil on success, or an error if a signal was received.
func (n *Node) Wait(duration time.Duration) error {
	if n.handle == nil {
		return ErrNodeClosed
	}

	secs := uint64(duration.Seconds())
	nanos := uint32(duration.Nanoseconds() % 1e9)

	result := C.iox2_node_wait(&n.handle, C.uint64_t(secs), C.uint32_t(nanos))
	if result != C.IOX2_OK {
		return NodeWaitError(result)
	}
	return nil
}

// nodeWaitContextPollInterval is the maximum duration each underlying C wait call
// will block before re-checking the context. The C API has no cancellable wait,
// so we slice the wait into fixed-size chunks to remain responsive to cancellation.
const nodeWaitContextPollInterval = 100 * time.Millisecond

// WaitWithContext waits until the context is done or a signal is received.
// This provides better integration with Go's context-based cancellation.
// The node is polled in nodeWaitContextPollInterval slices because the
// underlying C API does not support cancellable waits.
func (n *Node) WaitWithContext(ctx context.Context) error {
	if n.handle == nil {
		return ErrNodeClosed
	}

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			err := n.Wait(nodeWaitContextPollInterval)
			if err != nil {
				return err
			}
		}
	}
}

// ServiceBuilder returns a new ServiceBuilder for creating services associated with this Node.
func (n *Node) ServiceBuilder(serviceName *ServiceName) *ServiceBuilder {
	if n.handle == nil || serviceName == nil {
		return nil
	}

	handle := C.iox2_node_service_builder(&n.handle, nil, serviceName.ptr())
	return &ServiceBuilder{
		handle:      handle,
		serviceType: n.serviceType,
	}
}

// ServiceType returns the ServiceType of the Node.
func (n *Node) ServiceType() ServiceType {
	return n.serviceType
}

// ID returns the unique NodeId of this node.
func (n *Node) ID() *NodeId {
	if n.handle == nil {
		return nil
	}
	ptr := C.iox2_node_id(&n.handle, uint32(n.serviceType))
	if ptr == nil {
		return nil
	}
	// Convert the pointer to a handle
	// Note: The NodeId ptr is a borrowed reference; we don't own it
	return &NodeId{
		ptr: ptr,
	}
}

// SignalHandlingMode returns the signal handling mode with which the node was created.
func (n *Node) SignalHandlingMode() SignalHandlingMode {
	if n.handle == nil {
		return SignalHandlingModeDisabled
	}
	return SignalHandlingMode(C.iox2_node_signal_handling_mode(&n.handle))
}

// Config returns a pointer to the node's configuration.
// The returned pointer is valid for the lifetime of the Node.
func (n *Node) Config() *Config {
	if n.handle == nil {
		return nil
	}
	ptr := C.iox2_node_config(&n.handle)
	if ptr == nil {
		return nil
	}
	return &Config{ptr: ptr}
}

// Config represents the iceoryx2 configuration.
type Config struct {
	ptr C.iox2_config_ptr
}

// NodeId represents a unique identifier for a Node.
type NodeId struct {
	handle C.iox2_node_id_h
	ptr    C.iox2_node_id_ptr // borrowed pointer, not owned
}

// Close releases the resources associated with the NodeId.
// Implements io.Closer.
func (id *NodeId) Close() error {
	if id.handle != nil {
		C.iox2_node_id_drop(id.handle)
		id.handle = nil
	}
	// ptr is borrowed, don't drop it
	return nil
}

// Pid returns the process ID associated with this NodeId.
func (id *NodeId) Pid() int32 {
	if id.handle != nil {
		return int32(C.iox2_node_id_pid(&id.handle))
	}
	if id.ptr != nil {
		// Clone from ptr to get a handle, then get pid
		var tempHandle C.iox2_node_id_h
		C.iox2_node_id_clone_from_ptr(nil, id.ptr, &tempHandle)
		if tempHandle != nil {
			pid := int32(C.iox2_node_id_pid(&tempHandle))
			C.iox2_node_id_drop(tempHandle)
			return pid
		}
	}
	return 0
}

// NodeState represents the state of a Node in the system.
type NodeState int

const (
	NodeStateAlive        NodeState = C.iox2_node_state_e_ALIVE
	NodeStateDead         NodeState = C.iox2_node_state_e_DEAD
	NodeStateInaccessible NodeState = C.iox2_node_state_e_INACCESSIBLE
	NodeStateUndefined    NodeState = C.iox2_node_state_e_UNDEFINED
)

// String implements fmt.Stringer for NodeState.
func (s NodeState) String() string {
	switch s {
	case NodeStateAlive:
		return "Alive"
	case NodeStateDead:
		return "Dead"
	case NodeStateInaccessible:
		return "Inaccessible"
	case NodeStateUndefined:
		return "Undefined"
	default:
		return "Unknown"
	}
}

// NodeListCallback is called for each node during node listing.
type NodeListCallback func(state NodeState, nodeId *NodeId, name string) CallbackProgression

// NodeInfo contains information about a node found during listing.
type NodeInfo struct {
	State      NodeState
	Name       string
	Executable string
	Pid        int32
}

// ListNodes lists all nodes in the system matching the service type.
// Returns a slice of NodeInfo for each node found.
func ListNodes(serviceType ServiceType, config *Config) ([]NodeInfo, error) {
	var nodes []NodeInfo

	callback := func(state NodeState, nodeId *NodeId, name string) CallbackProgression {
		info := NodeInfo{
			State: state,
			Name:  name,
		}
		if nodeId != nil {
			info.Pid = nodeId.Pid()
		}
		nodes = append(nodes, info)
		return CallbackProgressionContinue
	}

	err := ListNodesWithCallback(serviceType, config, callback)
	return nodes, err
}

// ListNodesWithCallback lists all nodes in the system, calling the callback for each.
// The callback can return CallbackProgressionStop to stop the listing early.
func ListNodesWithCallback(serviceType ServiceType, config *Config, callback NodeListCallback) error {
	return listNodesImpl(serviceType, config, callback)
}

// DeadNodeView represents a dead node in the system.
type DeadNodeView struct {
	nodeId *NodeId
	config *Config
}

// RemoveStaleResources removes stale resources left behind by a dead node.
// Returns true if resources were successfully cleaned up.
func RemoveStaleResources(serviceType ServiceType, nodeId *NodeId, config *Config) (bool, error) {
	const op = "RemoveStaleResources"

	if nodeId == nil {
		return false, WrapError(op, fmt.Errorf("nodeId cannot be nil"))
	}

	// We need to create handles from the pointers/handles for the C API
	var nodeIdHandle C.iox2_node_id_h
	var configHandle C.iox2_config_h

	// Clone node_id to get a handle we own
	if nodeId.ptr != nil {
		C.iox2_node_id_clone_from_ptr(nil, nodeId.ptr, &nodeIdHandle)
	} else if nodeId.handle != nil {
		C.iox2_node_id_clone_from_handle(nil, &nodeId.handle, &nodeIdHandle)
	} else {
		return false, WrapError(op, fmt.Errorf("nodeId has no valid pointer or handle"))
	}
	defer C.iox2_node_id_drop(nodeIdHandle)

	// Get config pointer - use global config if none provided
	var configPtr C.iox2_config_ptr
	if config != nil && config.ptr != nil {
		configPtr = config.ptr
	} else {
		configPtr = C.iox2_config_global_config()
	}

	// Create config handle from pointer
	C.iox2_config_from_ptr(configPtr, nil, &configHandle)
	defer C.iox2_config_drop(configHandle)

	// Call the FFI function
	var hasSuccess C.bool
	result := C.iox2_dead_node_remove_stale_resources(
		uint32(serviceType),
		&nodeIdHandle,
		&configHandle,
		&hasSuccess,
	)

	if result != C.IOX2_OK {
		return false, WrapError(op, fmt.Errorf("failed with error code %d", result))
	}

	return bool(hasSuccess), nil
}
