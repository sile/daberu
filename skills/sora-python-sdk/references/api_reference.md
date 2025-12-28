# Sora Python SDK API Reference

## Core Classes

### Sora

Main SDK class for creating connections and sources.

```python
Sora(
    openh264: str | None = None,
    video_codec_preference: SoraVideoCodecPreference | None = None,
    force_i420_conversion: bool | None = None
)
```

**Methods**:

- `create_connection(...)` → `SoraConnection` - Create connection to Sora server
- `create_audio_source(channels: int, sample_rate: int)` → `SoraAudioSource` - Create audio source
- `create_video_source()` → `SoraVideoSource` - Create video source
- `create_libcamera_source(...)` → `SoraTrackInterface` - Create Raspberry Pi camera source (Pi only)

### SoraConnection

Manages connection to Sora server.

**Methods**:

- `connect()` - Establish connection
- `disconnect()` - Close connection
- `send_data_channel(label: str, data: bytes) → bool` - Send data channel message
- `get_stats() → str` - Get WebRTC statistics (JSON string)

**Callbacks**:

- `on_set_offer: Callable[[str], None]` - Called when offer received
- `on_notify: Callable[[str], None]` - Called for notify messages
- `on_disconnect: Callable[[SoraSignalingErrorCode, str], None]` - Called on disconnect
- `on_track: Callable[[SoraMediaTrack], None]` - Called when track received
- `on_data_channel: Callable[[str], None]` - Called when data channel ready
- `on_message: Callable[[str, bytes], None]` - Called for data channel messages
- `on_switched: Callable[[str], None]` - Called when switched to data channel signaling
- `on_ws_close: Callable[[int, str], None]` - Called when WebSocket closes

### SoraAudioSource

Sends audio to Sora.

**Methods**:

- `on_data(data: np.ndarray)` - Send audio frame (samples_per_channel × channels, int16)
- `on_data(data: np.ndarray, timestamp: float)` - Send with timestamp

**Audio Format**: 16-bit PCM, shape `(samples_per_channel, channels)`

### SoraVideoSource

Sends video to Sora.

**Methods**:

- `on_captured(frame: np.ndarray)` - Send video frame (H × W × 3 BGR, uint8)
- `on_captured(frame: np.ndarray, timestamp: float)` - Send with timestamp
- `on_captured(frame: np.ndarray, timestamp_us: int)` - Send with microsecond timestamp

**Video Format**: BGR format, shape `(height, width, 3)`, dtype `uint8`

### SoraAudioSink

Receives audio from Sora.

```python
SoraAudioSink(
    track: SoraMediaTrack,
    output_frequency: int,
    output_channels: int
)
```

**Methods**:

- `read(frames: int, timeout: float = 0.0) → tuple[bool, np.ndarray]` - Read audio data
  - Returns: `(success, audio_data)` where audio_data shape is `(frames, channels)`

### SoraVideoSink

Receives video from Sora.

```python
SoraVideoSink(track: SoraMediaTrack)
```

**Callbacks**:

- `on_frame: Callable[[SoraVideoFrame], None]` - Called for each video frame

### SoraVideoFrame

Contains received video frame.

**Methods**:

- `data() → np.ndarray` - Get frame as numpy array (H × W × 3 BGR)

### SoraAudioStreamSink

Receives audio as stream (10ms chunks).

```python
SoraAudioStreamSink(
    track: SoraMediaTrack,
    output_frequency: int,
    output_channels: int
)
```

**Callbacks**:

- `on_frame: Callable[[SoraAudioFrame], None]` - Called for each 10ms audio chunk

### SoraAudioFrame

Contains 10ms audio chunk.

**Methods**:

- `data() → np.ndarray` - Get audio as numpy array (samples × channels, int16)
- `samples_per_channel() → int` - Get number of samples per channel
- `num_channels() → int` - Get number of channels
- `sample_rate_hz() → int` - Get sample rate

## Video Codec Configuration

### SoraVideoCodecPreference

```python
SoraVideoCodecPreference(codecs: list[Codec])
```

**Codec**:
```python
Codec(
    type: SoraVideoCodecType,
    encoder: SoraVideoCodecImplementation | None = None,
    decoder: SoraVideoCodecImplementation | None = None
)
```

**SoraVideoCodecType**:
- `VP8`, `VP9`, `AV1`, `H264`, `H265`

**SoraVideoCodecImplementation**:
- `INTERNAL` - Software codec
- `INTEL_VPL` - Intel hardware
- `NVIDIA_VIDEO_CODEC_SDK` - NVIDIA hardware
- `APPLE_VIDEO_TOOLBOX` - Apple hardware
- `AMD_AMF` - AMD hardware
- `RASPI_V4L2M2M` - Raspberry Pi hardware
- `CISCO_OPENH264` - OpenH264 software

**Helper Functions**:

- `get_video_codec_capability(openh264: str | None = None) → SoraVideoCodecCapability`
- `create_video_codec_preference_from_implementation(capability, implementation) → SoraVideoCodecPreference`

## Encoded Transform

### SoraVideoFrameTransformer

```python
transformer = SoraVideoFrameTransformer()
transformer.on_transform = lambda frame: ...
transformer.enqueue(frame)
```

### SoraAudioFrameTransformer

```python
transformer = SoraAudioFrameTransformer()
transformer.on_transform = lambda frame: ...
transformer.enqueue(frame)
```

### SoraTransformableVideoFrame

Encoded video frame for transformation.

**Methods**:

- `get_data() → np.ndarray` - Get encoded data
- `set_data(data: np.ndarray)` - Set encoded data
- `get_ssrc() → int` - Get SSRC
- `get_timestamp() → int` - Get RTP timestamp
- `is_key_frame() → bool` - Check if keyframe

### SoraTransformableAudioFrame

Encoded audio frame for transformation.

**Methods**:

- `get_data() → np.ndarray` - Get encoded data
- `set_data(data: np.ndarray)` - Set encoded data
- `get_ssrc() → int` - Get SSRC
- `get_timestamp() → int` - Get RTP timestamp

## Voice Activity Detection

### SoraVAD

```python
vad = SoraVAD()
probability = vad.analyze(audio_frame)  # Returns 0.0-1.0
```

Returns probability that audio frame contains voice. Use threshold of 0.95 (same as libwebrtc).

## Connection Parameters

### create_connection() Parameters

**Required**:
- `signaling_urls: list[str]` - Sora signaling URLs
- `role: str` - "sendonly", "recvonly", or "sendrecv"
- `channel_id: str` - Channel ID

**Common Optional**:
- `metadata: dict | None` - Metadata (e.g., auth tokens)
- `audio: bool | None` - Enable audio (default: True)
- `video: bool | None` - Enable video (default: True)
- `audio_source: SoraAudioSource | None` - Audio source for sending
- `video_source: SoraVideoSource | None` - Video source for sending
- `video_codec_type: str | None` - "VP8", "VP9", "AV1", "H264", "H265"
- `video_bit_rate: int | None` - Video bitrate in kbps
- `audio_bit_rate: int | None` - Audio bitrate in kbps
- `simulcast: bool | None` - Enable simulcast
- `data_channels: list[dict] | None` - Data channel configurations
- `data_channel_signaling: bool | None` - Use data channel for signaling

**Transform**:
- `audio_frame_transformer: SoraAudioFrameTransformer | None`
- `video_frame_transformer: SoraVideoFrameTransformer | None`

**Advanced**:
- `spotlight: bool | None` - Enable spotlight
- `spotlight_number: int | None` - Number of spotlight streams
- `forwarding_filter: dict | None` - Forwarding filter rules
- `degradation_preference: SoraDegradationPreference | None` - Quality degradation preference

## Enums

### SoraSignalingErrorCode

Error codes for disconnection:
- `CLOSE_SUCCEEDED`
- `INTERNAL_ERROR`
- `INVALID_PARAMETER`
- `WEBSOCKET_HANDSHAKE_FAILED`
- `WEBSOCKET_ONCLOSE`
- `WEBSOCKET_ONERROR`
- `PEER_CONNECTION_STATE_FAILED`
- `ICE_FAILED`

### SoraDegradationPreference

Quality degradation when bandwidth limited:
- `MAINTAIN_FRAMERATE` - Reduce resolution
- `MAINTAIN_RESOLUTION` - Reduce framerate
- `BALANCED` - Reduce both
- `DISABLED` - No adaptation

## Data Channel Configuration

```python
data_channels = [
    {
        "label": "#messaging",
        "direction": "sendrecv",  # or "sendonly", "recvonly"
        "ordered": True,
        "compress": False
    }
]
```

## Statistics Format

`get_stats()` returns JSON string with WebRTC stats. Parse with `json.loads()`.

Common stat types:
- `"codec"` - Codec information
- `"inbound-rtp"` - Receiving stats
- `"outbound-rtp"` - Sending stats
- `"transport"` - Connection stats
- `"data-channel"` - Data channel stats

Example:
```python
stats = json.loads(connection.get_stats())
for stat in stats:
    if stat["type"] == "outbound-rtp" and stat["kind"] == "video":
        print(f"Video sent: {stat['bytesSent']} bytes")
        print(f"Encoder: {stat.get('encoderImplementation')}")
```
