# Raspi OS

A Raspberry Pi operating system.

## Background

The [Raspberry Pi](https://en.wikipedia.org/wiki/Raspberry_Pi) is a single board ARM computer equipped with modern peripherals as well as GPIO pins. Typically, it is used in conjunction with [Raspbian](https://en.wikipedia.org/wiki/Raspberry_Pi_OS) or another Linux distro, but since many of the components are open source, it is possible to write bare metal programs that are loaded directly by the firmware on startup.

### Definitions

Since embedded development has a jargon of its own, especially with the use of abbreviations, a table of common ones is included below.
| Abbreviation | Term | Definition |
| :-: | :-: | :-: |
| **ARM** | [**A**dvanced **R**isc **M**achine](https://en.wikipedia.org/wiki/ARM_architecture_family) | A Processor Architecture designed on the principles of reduced instruction set computing |
| **EL**, **EL**n | [**E**xception **L**evel](https://developer.arm.com/documentation/102412/0102/Privilege-and-Exception-levels) | A hardware [CPU mode](https://en.wikipedia.org/wiki/CPU_modes) that enforces different privelege levels |
| **GPIO** | [**G**eneral **P**urpose **I**nput/**O**utput](https://en.wikipedia.org/wiki/General-purpose_input/output) | A hardware pin that may be controlled at runtime that is not pre-assigned a specific purpose. |
| **MMIO** | [**M**emory-**M**apped **I**/**0**](https://en.wikipedia.org/wiki/Memory-mapped_I/O) | Hardware peripherals that are interfaced using registers that are mapped to the same address space as the system memory |
| **MMU** | [**M**emory **M**anagement **U**nit](https://en.wikipedia.org/wiki/Memory_management_unit) | A processor unit that controls caching and address translation for the system memory |
| **UART** | [**U**niversal **A**synchronous **R**eceiver-**T**ransmitter](https://en.wikipedia.org/wiki/Universal_asynchronous_receiver-transmitter) | A protocol which allows for bidirectional asynchronous communication between to parties.

## Outline

### User Input

Currently there are no input devices in development, however in the future buttons (easy) or usb devices (hard) may be used.

### Output to User

The user can receive output via a LED status light, a UART cable, or a monitor that is connected to the Raspberry Pi.

### Hardware Abstraction

When working with the low level features of the Pi, we should take care to separate the software interface from the hardware implementation. As a matter of good practice, this will give us more flexibility in the future, especially as more components become involved.

### Kernel

The purpose of the kernel is to load and run other programs and serve as an abstraction layer between the hardware features and the software implementations.

### Benchmarking
While common sense and theoretical models can provide guidance for design choices and component implementations, we should rely on runtime data as the ultimate arbiter of performance. To do this, we should write benchmarks for any potentially expensive algorithms and functions to have meaningful data on performance.

## Output to User

The Raspberry Pi's IO capabilities are detailed in the aptly named [BCM2835 ARM Peripherals](https://www.raspberrypi.org/app/uploads/2012/02/BCM2835-ARM-Peripherals.pdf) document.

### GPIO

The GPIO hardware of the Raspberry Pi 3 is described in part 6 of the BCM Peripherals documentation. The register file for the GPIO starts at an offset of `0x0020_0000` from the base MMIO address. Each pin has up to 8 possible functions which can be selected using a GPIO Function Select (`GPFSEL`) register. For our purposes, we can focus mainly on the output function. Once flagged as an output, each pin can be set high or cleared using its `GPSET` and `GPCLR` registers.

### UART
The Raspberry Pi implementation of UART is described in part 2 of the BCM documentation. Once connected, it allows for text communication between the Raspberry Pi and a connected computer which we can use for debugging and logging purposes.

### Display
Using the [Mailbox Property](https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface) interface, we can request a frame buffer from the Raspberry Pi VC GPU and treat it as a 2D array, with each index corresponding to a pixel.

## Hardware Abstraction
The main focus for our abstraction efforts is our API for interacting with hardware. As the hardware components become more complex, they begin to rely more on other components. For example, the UART controller relies on the MMIO controller, and GPIO controller, which itself relies on the MMIO controller again. By encapsulating each of these separate responsibilities, that is UART, GPIO, and MMIO in separate objects, we can create a logging interface that feels rather platform agnostic. The downside, of course, is that there are occasional repeated references. Such repetition is not enough to abandon our efforts at clean code, since the additional references do not take significantly more data and the occasional repetition is well worth the cleaner project structure.

## Kernel

### Privilege and Exception Levels
ARM, as a modern architecture, provides a hardware mechanism for managing the privileges of programs by providing 4 exception levels, `EL3`-`EL0`, with the higher number indicating increased privilege. Typically, kernels run in `EL1` while user-facing software runs in `EL0`. When our program is loaded it starts in `EL2`. For our purposes, we should enter `EL1`, especially for the purposes of configuring the memory management unit.

As the ARM documentation explains:
> The current level of privilege can only change when the processor takes or returns from an exception. Therefore,  these privilege levels are referred to as Exception levels in the Arm architecture.

This unfortunate naming scheme results in a mechanism for chaning exception levels that seems rather hacky. An exception is "simulated" by populating the Saved Program Status Register (`spsr_el2`), and the Hypervisor Control Register (`hcr_el2`) with the values they would have on an actual exception and then pointing the Exception Link Register (`elr_el2`) to the target start of execution in `EL1`. Once this is done, we can "return" to `EL1` using the exception return (`eret`) instruction. Optionally, other registers can be populated with values to allow `EL1` programs to access certain processor features such as the FPU. This feels a little less hacky, however, if we think of the exception link register and exception return as analogs of the link register (`lr`) and return ('ret') instruction and a change in exception level as another type of branch.

Further documentation can be found on the [ARM Website](https://developer.arm.com/documentation/102412/0102/Privilege-and-Exception-levels).

## MMU
ARMS offers documentation for its MMU on [its website](https://documentation-service.arm.com/static/5efa1d23dbdee951c1ccdec5?token=).