# Computation

`aspect.computation` records one or more computations performed by a research activity. Each computation contains stages in dependency order. A stage is classical or uses a specified quantum execution model, and its input and output ports reuse the existing ASPECT `Data` description. Algorithm complexity describes the work independently of any machine.

An execution profile connects a computation to an environment in `meta.infrastructure`. Each stage execution names a platform in that environment. Profiles can record workload variable values, operation duration, repetitions, and an observed end-to-end duration. A compiled circuit's physical depth belongs in its stage execution; a circuit stage's logical depth belongs in the computation definition.

Compute impedance is the elapsed overhead around the operations. Profile costs describe input and output transfer, compilation, queueing, setup, state preparation, synchronization, readout, and postprocessing. Each cost says whether it was measured or estimated and whether it applies to the whole workload, a batch, an invocation, or a repetition. A transfer cost can name its stage and input or output port, and include volume in bytes, effective bandwidth in bytes per second, and latency in milliseconds. Unknown values are omitted, not interpreted as zero.

The observed end-to-end duration is recorded independently because stages and transfers can overlap. A one-second operation with a sixty-second upload can therefore have a sixty-one-second observed duration. `acorn-schema` stores the observations; any prediction or bottleneck calculation belongs in `acorn-lib`.

Quantum hardware in `hardware::Resource::Quantum` describes the processor's model, physical technology, connectivity, physical and logical qubit capacities, and error correction. A quantum stage describes the algorithm and its circuit, annealing, analog, or measurement-based requirements. `standard::ocp` continues to describe authoritative OCP records associated with infrastructure rather than the computation itself.
