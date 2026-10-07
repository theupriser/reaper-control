"""Generates click.wav: 60 s, 120 BPM, 8 kHz mono. A 5 ms 2 kHz burst every beat, accented (3 kHz; 4 kHz is silent at 8 kHz sampling) on the bar."""
import math, struct, wave

RATE, SECONDS, BPM = 8000, 60, 120
samples = [0] * (RATE * SECONDS)
beat = 60 / BPM
for n in range(int(SECONDS / beat)):
    freq = 3000 if n % 4 == 0 else 2000
    start = int(n * beat * RATE)
    for i in range(int(0.005 * RATE)):
        if start + i < len(samples):
            samples[start + i] = int(8000 * math.sin(2 * math.pi * freq * i / RATE))
with wave.open("click.wav", "wb") as f:
    f.setnchannels(1)
    f.setsampwidth(2)
    f.setframerate(RATE)
    f.writeframes(b"".join(struct.pack("<h", s) for s in samples))
