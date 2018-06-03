# COST

COST is an acronym for the "Configuration that Outperforms a Single Thread", indicating the hardware resources required by a distributed system before it begins to outperform a single-threaded implementation. This repository contains the single-threaded implementations providing the baseline performance.

Specifically, this repository contains single-threaded implementations of three graph algorithms, [PageRank](http://en.wikipedia.org/wiki/PageRank), [label propagation](http://www.cs.cmu.edu/~ukang/papers/HalfpICDE2011.pdf), and [union-find](http://en.wikipedia.org/wiki/Disjoint-set_data_structure), supporting performance measurements taken on two graphs, [twitter_rv](http://an.kaist.ac.kr/traces/WWW2010.html) and [uk_2007_05](http://law.di.unimi.it/webdata/uk-2007-05/). The code is intended to be instructive, rather than a meaningful replacement for a graph-processing system.

## Instructions

The project consists of several independent binaries and a supporting library. The `src/bin/` directory has one file for each binary, each of which can be executed by typing

    cargo run --release --bin <binary_name> -- <arguments>

All binaries take at least one argument, and can be run with zero arguments to present their usage (and any warnings about files that may be overwritten as a result of executing the binary).

### Introducing graph data

The most common first binary to use is `to_vertex`, which creates a binary representation of data presented as a textual list of pairs of vertex identifiers (one per line). You should be able to type:

    % cargo run --release --bin to_vertex
        Finished release [optimized] target(s) in 0.0 secs
         Running `target/release/to_vertex`
    Usage: to_vertex <source> <prefix>
    NOTE: <prefix>.nodes and <prefix>.edges will be overwritten.
    %

If you acquire some excellent graph data, you could for example type

    % cargo run --release --bin to_vertex -- my_graph.txt my_graph

which will create files `my_graph.nodes` and `my_graph.edges`. These files will generally be smaller than the textual representation, though the `.nodes` file will use space proportional to the largest vertex identifier.

Once you have ingressed some graph data, you can also re-arrange the data according to a Hilbert curve, which is an excellent bit of mathematics you can search for and read about if you so care.

    % cargo run --release --bin to_hilbert -- my_graph

will produce `my_graph.upper` and `my_graph.lower` for pre-existing `my_graph.nodes` and `my_graph.edges`. The Hilbert representation can be even a bit tighter, and often has improved performance for several of the algorithms.

### Graph algorithms

There are three algorithms here: pagerank, label propagation, and union find. Each has their own binary, and each expects you to supply three arguments: the "mode", which is one of `vertex`, `hilbert`, and `compressed`, the graph filename prefix, and a number greater than the largest vertex identifier (a size for per-vertex state allocation). If you don't know the last number, the `stats` binary can help you out by scanning the graph for you.

For example,

    % cargo run --release --bin union_find -- hilbert ./friendster 66000000
        Finished release [optimized] target(s) in 0.0 secs
         Running `target/release/union_find hilbert ./friendster 66000000`
    65608365 non-roots found
    %

which reports the number of nodes in the graph minus the number of connected components.

## Notes

There is a [companion COST repository](https://github.com/MicrosoftResearch/NaiadSamples) managed by Microsoft Research, including the state of the project several months ago. This may be helpful if you are interested in the corresponding C# implementations. The repository also contains [Naiad](http://research.microsoft.com/Naiad/) implementations that were done more recently. I am no longer affiliated with Microsoft and cannot commit to the repository (nor, historically, do they accept pull requests), and must apologize for the sorry state I left the code in. It may be cleaned up in the future (either by me, or other more industrious souls), given the right incentives.
