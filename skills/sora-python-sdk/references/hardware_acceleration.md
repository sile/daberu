# Hardware Acceleration Setup

## Overview

Sora Python SDK supports multiple hardware acceleration platforms for video encoding/decoding.

## Platform Support

| Platform | Codecs | Implementation |
|----------|--------|----------------|
| Intel VPL | VP9, AV1, H264, H265 | `INTEL_VPL` |
| NVIDIA Video Codec SDK | VP8/VP9 (decode), AV1, H264, H265 | `NVIDIA_VIDEO_CODEC_SDK` |
| Apple VideoToolbox | H264, H265 | `APPLE_VIDEO_TOOLBOX` |
| AMD AMF | VP9 (decode), AV1, H264, H265 | `AMD_AMF` |
| Raspberry Pi | H264 | `RASPI_V4L2M2M` |
| OpenH264 | H264 | `CISCO_OPENH264` |

## Intel VPL (Ubuntu/Windows)

**Supported Codecs**: VP9, AV1, H264, H265

**Requirements**:
- Intel GPU with hardware encoding/decoding support
- libvpl drivers installed

**Ubuntu Setup**:
```bash
# Install Intel Media drivers
sudo apt-get install intel-media-va-driver-non-free
```

**Usage**:
```python
from sora_sdk import (
    Sora,
    SoraVideoCodecPreference,
    SoraVideoCodecType,
    SoraVideoCodecImplementation
)

preference = SoraVideoCodecPreference(
    codecs=[
        SoraVideoCodecPreference.Codec(
            type=SoraVideoCodecType.H264,
            encoder=SoraVideoCodecImplementation.INTEL_VPL,
            decoder=SoraVideoCodecImplementation.INTEL_VPL
        )
    ]
)

sora = Sora(video_codec_preference=preference)
```

## NVIDIA Video Codec SDK (Ubuntu/Windows)

**Supported Codecs**: 
- Decode: VP8, VP9, AV1, H264, H265
- Encode: AV1, H264, H265

**Requirements**:
- NVIDIA GPU with NVENC/NVDEC support
- NVIDIA drivers installed

**Usage**:
```python
preference = SoraVideoCodecPreference(
    codecs=[
        SoraVideoCodecPreference.Codec(
            type=SoraVideoCodecType.H264,
            encoder=SoraVideoCodecImplementation.NVIDIA_VIDEO_CODEC_SDK,
            decoder=SoraVideoCodecImplementation.NVIDIA_VIDEO_CODEC_SDK
        )
    ]
)

sora = Sora(video_codec_preference=preference)
```

## Apple VideoToolbox (macOS)

**Supported Codecs**: H264, H265

**Requirements**:
- macOS on Apple Silicon or Intel Mac

**Usage**:
```python
preference = SoraVideoCodecPreference(
    codecs=[
        SoraVideoCodecPreference.Codec(
            type=SoraVideoCodecType.H264,
            encoder=SoraVideoCodecImplementation.APPLE_VIDEO_TOOLBOX,
            decoder=SoraVideoCodecImplementation.APPLE_VIDEO_TOOLBOX
        )
    ]
)

sora = Sora(video_codec_preference=preference)
```

## AMD AMF (Ubuntu/Windows)

**Supported Codecs**: 
- Decode: VP9, AV1, H264, H265
- Encode: AV1, H264, H265

**Requirements**:
- AMD GPU with AMF support
- AMD drivers installed

**Usage**:
```python
preference = SoraVideoCodecPreference(
    codecs=[
        SoraVideoCodecPreference.Codec(
            type=SoraVideoCodecType.H264,
            encoder=SoraVideoCodecImplementation.AMD_AMF,
            decoder=SoraVideoCodecImplementation.AMD_AMF
        )
    ]
)

sora = Sora(video_codec_preference=preference)
```

## Raspberry Pi (Raspberry Pi OS)

**Supported Codecs**: H264

**Requirements**:
- Raspberry Pi 2/3/4/5/Zero 2 W
- Raspberry Pi OS (64-bit)
- Install `sora-sdk-rpi` package

**Installation**:
```bash
uv add sora-sdk-rpi
```

**Usage**:
```python
preference = SoraVideoCodecPreference(
    codecs=[
        SoraVideoCodecPreference.Codec(
            type=SoraVideoCodecType.H264,
            encoder=SoraVideoCodecImplementation.RASPI_V4L2M2M,
            decoder=SoraVideoCodecImplementation.RASPI_V4L2M2M
        )
    ]
)

sora = Sora(video_codec_preference=preference)
```

**Libcamera Source** (Raspberry Pi only):
```python
sora = Sora()

# Create camera source with libcamera
video_source = sora.create_libcamera_source(
    width=1920,
    height=1080,
    fps=30,
    native_frame_output=True,
    controls=[("AfMode", "Auto")]
)

connection = sora.create_connection(
    signaling_urls=["wss://..."],
    role="sendonly",
    channel_id="my-channel",
    video_source=video_source
)
```

## OpenH264 (Software)

**Supported Codecs**: H264

**Requirements**:
- Download OpenH264 binary from Cisco

**Download**:
```bash
# Linux
wget https://github.com/cisco/openh264/releases/download/v2.4.1/libopenh264-2.4.1-linux64.7.so.bz2
bunzip2 libopenh264-2.4.1-linux64.7.so.bz2

# macOS
wget https://github.com/cisco/openh264/releases/download/v2.4.1/libopenh264-2.4.1-mac-arm64.dylib.bz2
bunzip2 libopenh264-2.4.1-mac-arm64.dylib.bz2

# Windows
# Download .dll from releases page
```

**Usage**:
```python
preference = SoraVideoCodecPreference(
    codecs=[
        SoraVideoCodecPreference.Codec(
            type=SoraVideoCodecType.H264,
            encoder=SoraVideoCodecImplementation.CISCO_OPENH264,
            decoder=SoraVideoCodecImplementation.CISCO_OPENH264
        )
    ]
)

sora = Sora(
    openh264="/path/to/libopenh264.so",
    video_codec_preference=preference
)
```

## Checking Available Hardware

```python
from sora_sdk import (
    get_video_codec_capability,
    SoraVideoCodecImplementation
)

# Get capability
capability = get_video_codec_capability()

# Check available engines and codecs
for engine in capability.engines:
    print(f"Engine: {engine.name}")
    for codec in engine.codecs:
        if codec.encoder and codec.decoder:
            print(f"  {codec.type}: encoder={codec.encoder}, decoder={codec.decoder}")

# Check specific implementation
intel_available = any(
    e.name == SoraVideoCodecImplementation.INTEL_VPL 
    for e in capability.engines
)
print(f"Intel VPL available: {intel_available}")
```

## Automatic Selection

Use helper function to automatically select all codecs from a specific implementation:

```python
from sora_sdk import (
    get_video_codec_capability,
    create_video_codec_preference_from_implementation,
    SoraVideoCodecImplementation
)

capability = get_video_codec_capability()

# Automatically use Intel VPL for all supported codecs
preference = create_video_codec_preference_from_implementation(
    capability,
    SoraVideoCodecImplementation.INTEL_VPL
)

sora = Sora(video_codec_preference=preference)
```

## Performance Tips

1. **Use hardware acceleration when available** - Reduces CPU usage significantly
2. **Match codec to content** - H264 for general use, H265 for high quality, VP9/AV1 for efficiency
3. **Check encoder stats** - Use `get_stats()` to verify hardware encoder is being used:
   ```python
   stats = json.loads(connection.get_stats())
   for stat in stats:
       if stat.get("type") == "outbound-rtp":
           print(f"Encoder: {stat.get('encoderImplementation')}")
   ```

## Troubleshooting

**Hardware encoder not used**:
- Check drivers are installed
- Verify hardware supports the codec
- Check `encoderImplementation` in stats
- Try reducing resolution/bitrate

**Quality issues**:
- Increase `video_bit_rate` parameter
- Use `degradation_preference` to control quality vs framerate tradeoff
- For simulcast, ensure total bitrate is sufficient for all layers

**Compatibility**:
- Intel VPL: Requires Intel GPU (11th gen+)
- NVIDIA: Requires GTX 900 series or newer
- Apple: Requires macOS 10.13+ 
- AMD: Requires Radeon RX 400 series or newer
