# fractal_rs
This is a port to Rust of my implementation of rendering a Mandelbrot set written long long time ago in C++.
# Running
```cargo +nightly run --release```
# Navigation
* ```Left mouse click``` Centers view on given position
* ```[``` Zoom out
* ```]``` Zoom in
* ```=``` Increase iterations
* ```-``` Decrease iterations
* ```1...9``` Set center of the window to some POI (6 is iteresting one)
* ```0``` Reset view
* ```space``` Dump current position to the console
# Features
 
- [x] simple version - one thread + double
- [x] multithreaded - unsafeCell + atomics to sync up with workers
- [x] multithreaded using Rayon
- [x] avx2 version + Rayon
- [x] avx512 (lack of CPU to test it, lol)
- [x] quadruple (double double) + Rayon:

<img src="https://github.com/szymek156/fractal_rs/blob/master/images/double.png" alt="drawing" width="300"/> <img src="https://github.com/szymek156/fractal_rs/blob/master/images/double-double.png" alt="drawing" width="300"/>
- [ ] refactor, because it starts to be a mess
- [ ] quadruple + avx2 + Rayon
- [ ] use rust crate with arbitrary precision
- [ ] cuda?
- [ ] using new algorithm to calculate
    - https://math.stackexchange.com/questions/939270/perturbation-of-mandelbrot-set-fractal
    - http://www.science.eclipse.co.uk/sft_maths.pdf  superfractaling maths K. I. Martin
    - https://mathr.co.uk/mandelbrot/perturbation.pdf
- [ ] arbitrary precision
- [ ] Julia
- [ ] create a video from the pass
- [ ] adaptive float type selection on the fly
    - start with the floats, jump to doubles, then go to quads. Depending on the zoom magnitude.
