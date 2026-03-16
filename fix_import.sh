#!/bin/bash
sed -i '' 's/use objc2::runtime::{AnyObject, ProtocolObject};/use objc2::AllocAnyThread;\nuse objc2::runtime::{AnyObject, ProtocolObject};/g' player/src/systemint/macos.rs
