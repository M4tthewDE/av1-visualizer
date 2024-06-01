# Links

Convert mp4 to ivf

`ffmpeg -i input.mp4 -c:v copy output.ivf`

Analyze .ivf files

https://pengbins.github.io/aomanalyzer.io/

AV1 Codec ISO Media File Format Binding

https://aomediacodec.github.io/av1-isobmff/

# aomdec

This is an attempt at documenting the steps of the reference decoder.
More specifically when decoding an ivf file created with the command above.

av1/av1_dx_iface.c
init_decoder()
frame_worker_hook()

av1/decoder/decoder.c
av1_receive_compressed_data()

av1/decoder/obu.c
aom_decode_frame_from_obus()
