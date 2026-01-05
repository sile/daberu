# MP4-RUST Crate Documentation

## Crate Summary

`shiguredo_mp4` is a Rust library for reading and writing MP4 files with a focus on embedded and no_std environments. This crate provides a comprehensive set of tools for encoding and decoding MP4 box structures, demultiplexing MP4 files to extract media samples, and multiplexing media samples into MP4 files.

### Key Use Cases and Benefits

- **Parse MP4 Files**: Decode MP4 box structures and extract metadata and media samples
- **Generate MP4 Files**: Construct valid MP4 files from audio and video samples
- **No Dependencies**: Zero external dependencies for maximum portability
- **no_std Support**: Works in embedded and resource-constrained environments
- **Sans I/O Design**: Separates I/O operations from processing logic for flexibility
- **Multi-Codec Support**: 
  - Audio: Opus, FLAC, AAC
  - Video: VP8, VP9, AV1, H.264 (AVC), H.265 (HEVC)

### Design Philosophy

The crate follows a sans I/O architecture where the library provides the logic for processing MP4 structures but delegates actual I/O operations to the caller. This design enables:

1. **Flexibility**: Users control how and when data is read/written
2. **Efficiency**: Avoids unnecessary copies and allows streaming processing
3. **Portability**: Works with any I/O backend (files, network, memory)
4. **Testability**: Easy to test without real I/O operations

---

## API Reference

### Core Types and Traits

#### `BaseBox` Trait

The fundamental trait implemented by all MP4 boxes.

```rust
pub trait BaseBox {
    fn box_type(&self) -> BoxType;
    fn is_unknown_box(&self) -> bool;
    fn children<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = &'a dyn BaseBox>>;
}
```

**Methods:**
- `box_type()`: Returns the 4-character or UUID box type identifier
- `is_unknown_box()`: Returns true if this is an unrecognized box type
- `children()`: Returns an iterator over child boxes

#### `Encode` and `Decode` Traits

Core traits for serialization and deserialization.

```rust
pub trait Encode {
    fn encode(&self, buf: &mut [u8]) -> Result<usize>;
    fn encode_to_vec(&self) -> Result<Vec<u8>>;
}

pub trait Decode: Sized {
    fn decode(buf: &[u8]) -> Result<(Self, usize)>;
    fn decode_at(buf: &[u8], offset: &mut usize) -> Result<Self>;
}
```

**Parameters:**
- `buf`: Buffer for encoded data
- `offset`: Current position in buffer (for `decode_at`)

**Returns:**
- `encode()`: Number of bytes written
- `decode()`: Tuple of (decoded value, bytes consumed)

**Errors:**
- `ErrorKind::InsufficientBuffer`: Buffer too small
- `ErrorKind::InvalidData`: Malformed data

#### `Mp4File<B>`

Top-level structure representing an entire MP4 file.

```rust
pub struct Mp4File<B = RootBox> {
    pub ftyp_box: FtypBox,
    pub boxes: Vec<B>,
}
```

**Fields:**
- `ftyp_box`: File type box (mandatory, always first)
- `boxes`: Top-level boxes following ftyp

### Box Types

#### File Structure Boxes

##### `FtypBox` - File Type Box

Specifies the file format version and compatibility.

```rust
pub struct FtypBox {
    pub major_brand: Brand,
    pub minor_version: u32,
    pub compatible_brands: Vec<Brand>,
}
```

##### `MoovBox` - Movie Box

Contains all metadata for the presentation.

```rust
pub struct MoovBox {
    pub mvhd_box: MvhdBox,
    pub trak_boxes: Vec<TrakBox>,
    pub unknown_boxes: Vec<UnknownBox>,
}
```

##### `MdatBox` - Media Data Box

Contains the actual media samples.

```rust
pub struct MdatBox {
    pub payload: Vec<u8>,
}
```

#### Track Structure Boxes

##### `TrakBox` - Track Box

Represents a single track (audio or video).

```rust
pub struct TrakBox {
    pub tkhd_box: TkhdBox,
    pub edts_box: Option<EdtsBox>,
    pub mdia_box: MdiaBox,
    pub unknown_boxes: Vec<UnknownBox>,
}
```

##### `StblBox` - Sample Table Box

Contains information about samples in a track.

```rust
pub struct StblBox {
    pub stsd_box: StsdBox,  // Sample descriptions
    pub stts_box: SttsBox,  // Time-to-sample
    pub stsc_box: StscBox,  // Sample-to-chunk
    pub stsz_box: StszBox,  // Sample sizes
    pub stco_or_co64_box: Either<StcoBox, Co64Box>,  // Chunk offsets
    pub stss_box: Option<StssBox>,  // Sync samples
    pub unknown_boxes: Vec<UnknownBox>,
}
```

#### Sample Entry Types

##### `SampleEntry` Enum

Describes codec-specific information for samples.

```rust
pub enum SampleEntry {
    Avc1(Avc1Box),    // H.264
    Hev1(Hev1Box),    // H.265 (hev1)
    Hvc1(Hvc1Box),    // H.265 (hvc1)
    Vp08(Vp08Box),    // VP8
    Vp09(Vp09Box),    // VP9
    Av01(Av01Box),    // AV1
    Opus(OpusBox),    // Opus audio
    Mp4a(Mp4aBox),    // AAC audio
    Flac(FlacBox),    // FLAC audio
    Unknown(UnknownBox),
}
```

**Methods:**
- `audio_channel_count()`: Returns channel count for audio tracks
- `audio_sample_rate()`: Returns sample rate for audio tracks
- `video_resolution()`: Returns (width, height) for video tracks

### Demultiplexing API

#### `Mp4FileDemuxer`

Extracts media samples from MP4 files in temporal order.

```rust
pub struct Mp4FileDemuxer { /* private fields */ }

impl Mp4FileDemuxer {
    pub fn new() -> Self;
    pub fn required_input(&self) -> Option<RequiredInput>;
    pub fn handle_input(&mut self, input: Input);
    pub fn tracks(&mut self) -> Result<&[TrackInfo], DemuxError>;
    pub fn next_sample(&mut self) -> Result<Option<Sample<'_>>, DemuxError>;
}
```

**Key Methods:**

- `required_input()`: Returns information about needed data
  - Returns `None` when initialization is complete
  
- `handle_input(input)`: Provides file data to the demuxer
  - Must satisfy the requirements from `required_input()`
  
- `tracks()`: Returns metadata for all tracks
  - **Error**: `DemuxError::InputRequired` if more data needed
  
- `next_sample()`: Returns the next sample in temporal order
  - Returns `None` when all samples consumed
  - **Error**: `DemuxError::InputRequired` if more data needed

#### `Input<'a>`

Provides file data to the demuxer.

```rust
pub struct Input<'a> {
    pub position: u64,
    pub data: &'a [u8],
}
```

**Fields:**
- `position`: Byte offset in the file where `data` begins
- `data`: Buffer containing file data

#### `RequiredInput`

Describes data needed by the demuxer.

```rust
pub struct RequiredInput {
    pub position: u64,
    pub size: Option<usize>,
}
```

**Fields:**
- `position`: Byte offset where data is needed
- `size`: Number of bytes needed (None = until EOF)

**Methods:**
- `to_input(data)`: Creates an `Input` from this requirement
- `is_satisfied_by(input)`: Checks if an `Input` satisfies this requirement

#### `Sample<'a>`

Represents an extracted media sample.

```rust
pub struct Sample<'a> {
    pub track: &'a TrackInfo,
    pub sample_entry: Option<&'a SampleEntry>,
    pub keyframe: bool,
    pub timestamp: u64,
    pub duration: u32,
    pub data_offset: u64,
    pub data_size: usize,
}
```

**Fields:**
- `track`: Track this sample belongs to
- `sample_entry`: Codec information (Some only when changed)
- `keyframe`: True for sync/key frames
- `timestamp`: Presentation timestamp (in track timescale units)
- `duration`: Sample duration (in track timescale units)
- `data_offset`: Byte offset of sample data in file
- `data_size`: Size of sample data in bytes

#### `TrackInfo`

Metadata about a media track.

```rust
pub struct TrackInfo {
    pub track_id: u32,
    pub kind: TrackKind,
    pub duration: u64,
    pub timescale: NonZeroU32,
}
```

### Multiplexing API

#### `Mp4FileMuxer`

Creates MP4 files by multiplexing media samples.

```rust
pub struct Mp4FileMuxer { /* private fields */ }

impl Mp4FileMuxer {
    pub fn new() -> Result<Self, MuxError>;
    pub fn with_options(options: Mp4FileMuxerOptions) -> Result<Self, MuxError>;
    pub fn initial_boxes_bytes(&self) -> &[u8];
    pub fn append_sample(&mut self, sample: &Sample) -> Result<(), MuxError>;
    pub fn finalize(&mut self) -> Result<&FinalizedBoxes, MuxError>;
    pub fn finalized_boxes(&self) -> Option<&FinalizedBoxes>;
}
```

**Key Methods:**

- `new()`: Creates a muxer with default options
  
- `with_options(options)`: Creates a muxer with custom options
  
- `initial_boxes_bytes()`: Returns initial file header bytes
  - Must be written before any sample data
  
- `append_sample(sample)`: Notifies muxer that sample data was written
  - **Error**: `MuxError::PositionMismatch` if offset doesn't match expected
  - **Error**: `MuxError::MissingSampleEntry` if first sample lacks entry
  - **Error**: `MuxError::AlreadyFinalized` if called after finalize
  
- `finalize()`: Completes MP4 construction
  - Returns box updates needed to complete the file
  - **Error**: `MuxError::AlreadyFinalized` if already finalized

#### `Mp4FileMuxerOptions`

Configuration for the muxer.

```rust
pub struct Mp4FileMuxerOptions {
    pub reserved_moov_box_size: usize,
    pub creation_timestamp: Duration,
}
```

**Fields:**
- `reserved_moov_box_size`: Bytes to reserve for faststart (default: 0)
- `creation_timestamp`: File creation time (default: UNIX epoch)

**Helper Function:**
```rust
pub fn estimate_maximum_moov_box_size(sample_count_per_track: &[usize]) -> usize;
```

#### `Sample` (Muxer)

Sample to add to the MP4 file.

```rust
pub struct Sample {
    pub track_kind: TrackKind,
    pub sample_entry: Option<SampleEntry>,
    pub keyframe: bool,
    pub timescale: NonZeroU32,
    pub duration: u32,
    pub data_offset: u64,
    pub data_size: usize,
}
```

**Fields:**
- `track_kind`: Audio or Video
- `sample_entry`: Codec info (required for first sample)
- `keyframe`: True for sync/key frames
- `timescale`: Time units per second
- `duration`: Sample duration (in timescale units)
- `data_offset`: Where sample data was written
- `data_size`: Size of sample data

**Important:** All samples in a track must use the same timescale.

#### `FinalizedBoxes`

Information needed to complete the MP4 file.

```rust
pub struct FinalizedBoxes { /* private fields */ }

impl FinalizedBoxes {
    pub fn is_faststart_enabled(&self) -> bool;
    pub fn moov_box_size(&self) -> usize;
    pub fn offset_and_bytes_pairs(&self) -> impl Iterator<Item = (u64, &[u8])>;
    pub fn moov_box(&self) -> &MoovBox;
}
```

**Methods:**
- `is_faststart_enabled()`: True if moov box is at file start
- `moov_box_size()`: Final size of moov box
- `offset_and_bytes_pairs()`: Iterator of (offset, data) to write
- `moov_box()`: Returns the constructed moov box

### Auxiliary Types

#### `SampleTableAccessor<T>`

Efficiently accesses sample information from StblBox.

```rust
impl<T: AsRef<StblBox>> SampleTableAccessor<T> {
    pub fn new(stbl_box: T) -> Result<Self, SampleTableAccessorError>;
    pub fn sample_count(&self) -> u32;
    pub fn chunk_count(&self) -> u32;
    pub fn get_sample(&self, sample_index: NonZeroU32) -> Option<SampleAccessor<'_, T>>;
    pub fn get_sample_by_timestamp(&self, timestamp: u64) -> Option<SampleAccessor<'_, T>>;
    pub fn get_chunk(&self, chunk_index: NonZeroU32) -> Option<ChunkAccessor<'_, T>>;
    pub fn samples(&self) -> impl Iterator<Item = SampleAccessor<'_, T>>;
    pub fn chunks(&self) -> impl Iterator<Item = ChunkAccessor<'_, T>>;
}
```

**Errors:**
- `InconsistentSampleCount`: stts/stsz/stsc have different sample counts
- `FirstChunkIndexIsNotOne`: First stsc entry doesn't start at 1
- `ChunkIndicesNotMonotonicallyIncreasing`: stsc indices not sorted

#### `SampleAccessor<'a, T>`

Provides access to individual sample information.

```rust
impl<'a, T: AsRef<StblBox>> SampleAccessor<'a, T> {
    pub fn index(&self) -> NonZeroU32;
    pub fn duration(&self) -> u32;
    pub fn timestamp(&self) -> u64;
    pub fn data_size(&self) -> u32;
    pub fn data_offset(&self) -> u64;
    pub fn is_sync_sample(&self) -> bool;
    pub fn sync_sample(&self) -> Option<Self>;
    pub fn chunk(&self) -> ChunkAccessor<'a, T>;
}
```

#### `ChunkAccessor<'a, T>`

Provides access to chunk information.

```rust
impl<'a, T: AsRef<StblBox>> ChunkAccessor<'a, T> {
    pub fn index(&self) -> NonZeroU32;
    pub fn offset(&self) -> u64;
    pub fn sample_entry(&self) -> &'a SampleEntry;
    pub fn sample_count(&self) -> u32;
    pub fn samples(&self) -> impl Iterator<Item = SampleAccessor<'_, T>>;
}
```

### Error Types

#### `Error`

Main error type for encoding/decoding operations.

```rust
pub struct Error {
    pub kind: ErrorKind,
    pub reason: String,
    pub location: &'static Location<'static>,
    pub box_type: Option<BoxType>,
}
```

#### `ErrorKind`

Classification of errors.

```rust
pub enum ErrorKind {
    InvalidInput,         // Invalid input parameters
    InvalidData,          // Corrupted or malformed data
    InsufficientBuffer,   // Buffer too small
    Unsupported,          // Unsupported feature or format
}
```

#### `DemuxError`

Errors during demultiplexing.

```rust
pub enum DemuxError {
    DecodeError(Error),
    SampleTableError(SampleTableAccessorError),
    InputRequired(RequiredInput),
}
```

#### `MuxError`

Errors during multiplexing.

```rust
pub enum MuxError {
    EncodeError(Error),
    PositionMismatch { expected: u64, actual: u64 },
    MissingSampleEntry { track_kind: TrackKind },
    AlreadyFinalized,
    TimescaleMismatch { track_kind: TrackKind, expected: NonZeroU32, actual: NonZeroU32 },
}
```

---

## Examples and Common Patterns

### Basic: Decode an MP4 File

```rust
use shiguredo_mp4::{Mp4File, Decode};

fn decode_mp4_file(data: &[u8]) -> Result<Mp4File, shiguredo_mp4::Error> {
    let (file, _bytes_read) = Mp4File::decode(data)?;
    
    println!("Major brand: {:?}", file.ftyp_box.major_brand);
    println!("Number of top-level boxes: {}", file.boxes.len());
    
    Ok(file)
}
```

### Basic: Encode an MP4 File

```rust
use shiguredo_mp4::{Mp4File, Encode};
use shiguredo_mp4::boxes::{FtypBox, Brand};

fn encode_mp4_file(file: &Mp4File) -> Result<Vec<u8>, shiguredo_mp4::Error> {
    let bytes = file.encode_to_vec()?;
    Ok(bytes)
}
```

### Demultiplexing: Extract All Samples

```rust
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use shiguredo_mp4::demux::{Mp4FileDemuxer, Input, DemuxError};

fn extract_samples(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut demuxer = Mp4FileDemuxer::new();
    let mut buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
    
    // Initialize demuxer by providing required data
    loop {
        match demuxer.required_input() {
            Some(required) => {
                file.seek(SeekFrom::Start(required.position))?;
                let size = required.size.unwrap_or(buffer.len());
                let read = file.read(&mut buffer[..size])?;
                
                let input = Input {
                    position: required.position,
                    data: &buffer[..read],
                };
                demuxer.handle_input(input);
            }
            None => break,
        }
    }
    
    // Get track information
    let tracks = demuxer.tracks()?;
    for track in tracks {
        println!("Track {}: {:?}, duration={}, timescale={}",
                 track.track_id, track.kind, track.duration, track.timescale);
    }
    
    // Extract samples
    while let Some(sample) = demuxer.next_sample()? {
        println!("Sample: track={}, timestamp={}, size={}, keyframe={}",
                 sample.track.track_id, sample.timestamp, 
                 sample.data_size, sample.keyframe);
        
        // Read sample data
        file.seek(SeekFrom::Start(sample.data_offset))?;
        let mut sample_data = vec![0u8; sample.data_size];
        file.read_exact(&mut sample_data)?;
        
        // Process sample_data...
    }
    
    Ok(())
}
```

### Demultiplexing: Load Entire File to Memory

```rust
use shiguredo_mp4::demux::{Mp4FileDemuxer, Input};

fn demux_from_memory(file_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut demuxer = Mp4FileDemuxer::new();
    
    // Provide entire file at once
    let input = Input {
        position: 0,
        data: file_data,
    };
    demuxer.handle_input(input);
    
    let tracks = demuxer.tracks()?;
    println!("Found {} tracks", tracks.len());
    
    let mut sample_count = 0;
    while let Some(sample) = demuxer.next_sample()? {
        sample_count += 1;
        
        // Access sample data directly
        let sample_data = &file_data[sample.data_offset as usize..
                                     sample.data_offset as usize + sample.data_size];
        // Process sample_data...
    }
    
    println!("Processed {} samples", sample_count);
    Ok(())
}
```

### Multiplexing: Create an MP4 File

```rust
use std::fs::File;
use std::io::{Write, Seek, SeekFrom};
use std::num::NonZeroU32;
use shiguredo_mp4::mux::{Mp4FileMuxer, Sample};
use shiguredo_mp4::{TrackKind, boxes::SampleEntry};

fn create_mp4_file(
    output_path: &str,
    video_samples: Vec<(SampleEntry, Vec<u8>, bool)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut muxer = Mp4FileMuxer::new()?;
    
    // Write initial boxes
    let mut file = File::create(output_path)?;
    file.write_all(muxer.initial_boxes_bytes())?;
    
    let mut next_offset = muxer.initial_boxes_bytes().len() as u64;
    let timescale = NonZeroU32::new(30).unwrap(); // 30 fps
    
    // Add samples
    for (i, (sample_entry, data, is_keyframe)) in video_samples.into_iter().enumerate() {
        // Write sample data
        file.write_all(&data)?;
        
        // Notify muxer
        let sample = Sample {
            track_kind: TrackKind::Video,
            sample_entry: if i == 0 { Some(sample_entry) } else { None },
            keyframe: is_keyframe,
            timescale,
            duration: 1,
            data_offset: next_offset,
            data_size: data.len(),
        };
        muxer.append_sample(&sample)?;
        
        next_offset += data.len() as u64;
    }
    
    // Finalize
    let finalized = muxer.finalize()?;
    
    // Write final boxes
    for (offset, bytes) in finalized.offset_and_bytes_pairs() {
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(bytes)?;
    }
    
    println!("Created MP4 file at {}", output_path);
    println!("Faststart: {}", finalized.is_faststart_enabled());
    
    Ok(())
}
```

### Multiplexing: Faststart Mode

```rust
use shiguredo_mp4::mux::{Mp4FileMuxer, Mp4FileMuxerOptions, estimate_maximum_moov_box_size};
use std::time::Duration;

fn create_faststart_mp4() -> Result<(), Box<dyn std::error::Error>> {
    // Estimate moov box size
    let sample_counts = vec![150, 300]; // video: 150 samples, audio: 300 samples
    let estimated_size = estimate_maximum_moov_box_size(&sample_counts);
    
    let options = Mp4FileMuxerOptions {
        reserved_moov_box_size: estimated_size,
        creation_timestamp: Duration::from_secs(1234567890),
    };
    
    let mut muxer = Mp4FileMuxer::with_options(options)?;
    
    // ... add samples ...
    
    let finalized = muxer.finalize()?;
    assert!(finalized.is_faststart_enabled());
    
    Ok(())
}
```

### Working with Sample Tables

```rust
use shiguredo_mp4::aux::SampleTableAccessor;
use shiguredo_mp4::boxes::StblBox;

fn analyze_track_samples(stbl_box: &StblBox) -> Result<(), Box<dyn std::error::Error>> {
    let accessor = SampleTableAccessor::new(stbl_box)?;
    
    println!("Total samples: {}", accessor.sample_count());
    println!("Total chunks: {}", accessor.chunk_count());
    
    // Iterate through all samples
    for sample in accessor.samples() {
        println!("Sample {}: timestamp={}, duration={}, size={}, keyframe={}",
                 sample.index(),
                 sample.timestamp(),
                 sample.duration(),
                 sample.data_size(),
                 sample.is_sync_sample());
    }
    
    // Find sample by timestamp
    let timestamp = 1000;
    if let Some(sample) = accessor.get_sample_by_timestamp(timestamp) {
        println!("Sample at timestamp {}: index={}", 
                 timestamp, sample.index());
    }
    
    // Iterate through chunks
    for chunk in accessor.chunks() {
        println!("Chunk {}: offset={}, samples={}",
                 chunk.index(),
                 chunk.offset(),
                 chunk.sample_count());
    }
    
    Ok(())
}
```

### Error Handling Patterns

```rust
use shiguredo_mp4::{Decode, Error, ErrorKind};

fn handle_decode_errors(data: &[u8]) {
    match Mp4File::decode(data) {
        Ok((file, size)) => {
            println!("Successfully decoded {} bytes", size);
        }
        Err(e) => {
            match e.kind {
                ErrorKind::InvalidInput => {
                    eprintln!("Invalid input: {}", e.reason);
                }
                ErrorKind::InvalidData => {
                    eprintln!("Corrupted data: {}", e.reason);
                    if let Some(box_type) = e.box_type {
                        eprintln!("Error in box: {}", box_type);
                    }
                }
                ErrorKind::InsufficientBuffer => {
                    eprintln!("Buffer too small");
                }
                ErrorKind::Unsupported => {
                    eprintln!("Unsupported feature: {}", e.reason);
                }
            }
            eprintln!("Error location: {}", e.location);
        }
    }
}
```

### Edge Case: Handling Timestamp Gaps

When multiplexing, if there are gaps in timestamps, you need to handle them explicitly:

```rust
// For video: adjust the duration of the previous sample
let gap_duration = 5; // 5 frame units
let previous_sample_duration = 1 + gap_duration;

// For audio: insert silence to fill the gap
let silence_data = vec![0u8; gap_size];
// Write silence and add as a sample
```

### Edge Case: Large Files (>4GB)

```rust
use shiguredo_mp4::boxes::{Co64Box, Either};

// The muxer automatically uses co64 (64-bit offsets) when needed
fn check_offset_box(stbl_box: &StblBox) {
    match &stbl_box.stco_or_co64_box {
        Either::A(stco) => {
            println!("Using 32-bit offsets (stco)");
        }
        Either::B(co64) => {
            println!("Using 64-bit offsets (co64) for large file");
        }
    }
}
```

### Working with Unknown Boxes

```rust
use shiguredo_mp4::{Mp4File, BaseBox};
use shiguredo_mp4::boxes::UnknownBox;

fn traverse_boxes(file: &Mp4File) {
    let mut stack: Vec<&dyn BaseBox> = file.iter().collect();
    
    while let Some(b) = stack.pop() {
        if b.is_unknown_box() {
            println!("Unknown box type: {}", b.box_type());
        } else {
            println!("Known box type: {}", b.box_type());
        }
        
        // Traverse children
        stack.extend(b.children());
    }
}
```
