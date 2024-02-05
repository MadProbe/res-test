This is a tool to test out flash drives that I have used to test out mine if it has advertised data capacity.

To install this tool, download rust compiler and install nightly toolchain and run this project with `$env:RUSTFLAGS='-C target-cpu=native -C opt-level=3'; cargo build -r` in powershell window.
The help message embedded:
`Hey, we didn't do anything! Try specifing drive with --drive CLI argument lie this: --drive G: and try using --read-test argument to the CLI if you want to test a flash drive for data coherency or pass --write-test argument to the CLI if you want to test flash drive if it can write data to all the space it advertises it has or you can do --full-test if you want to do both`

This rep only work on x86 CPUs with [SSE4.1](https://en.wikipedia.org/wiki/SSE4?useskin=vector#SSE4.1) & [AES-NI](https://en.wikipedia.org/wiki/AES_instruction_set?useskin=vector#x86_architecture_processors) instructions support (and will work faster if you CPU supports AVX instructions).
Also if you exited early you would need to delete `res-test-stuff.bin` file and this program will try consume **ALL** the space on the disk (without remainder of division of free soace in disk by 4096).

This is also my initial try to program in rust btw.
