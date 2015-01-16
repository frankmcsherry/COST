# COST

COST is an acronym for the "Configuration that Outperforms a Single Thread", indicating the hardware resources required by a distributed system before it begins to outperform a single-threaded implementation. This repository contains the single-threaded implementations providing the baseline performance.

Specifically, this repository contains single-threaded implementations of three graph algorithms, [PageRank](http://en.wikipedia.org/wiki/PageRank), [label propagation](http://www.cs.cmu.edu/~ukang/papers/HalfpICDE2011.pdf), and [union-find](http://en.wikipedia.org/wiki/Disjoint-set_data_structure), supporting performance measurements taken on two graphs, [twitter_rv](http://an.kaist.ac.kr/traces/WWW2010.html) and [uk_2007_05](http://law.di.unimi.it/webdata/uk-2007-05/). The code is intended to be instructive, rather than a meaningful replacement for a graph-processing system.

## Instructions

Once cloned, the project can be built and run by typing
```
cargo run --release
```
which should result in usage information indicating appropriate arguments:
```
Running `target/COST`
Invalid arguments.

Usage: COST pagerank  (vertex | hilbert) <prefix>
       COST label_prop (vertex | hilbert) <prefix>
       COST union_find (vertex | hilbert) <prefix>
       COST to_hilbert [--dense] <prefix>
```
The first three modes correspond to the three graph algorithms. The second parameter indicates the binary graph layout. The final parameter must be a path prefix for which certain extensions exist as files, discussed in a moment. The fourth mode performs a graph layout according to a Hilbert space-filling curve, and prints the results to the screen (you must edit the code to write the data back to your computer).

## Graph input
The computation will not do anything productive without graph data, and the graph data use for experiments (processed versions of the graphs linked above) are too large to host here. I'm also not wild about distributing programs that write data back to someone else's computer, without more serious review. That being said, the file `src/twitter_parser.rs` contains the code that I used to parse `twitter_rv.net`, the file you get from the link above.

The required graph layout is quite simple (as is the code to parse it), and you should be able to write out your own graph data if you would like to try out the code.

The `vertex` option requires a `<prefix>` for which files `<prefix>.nodes` and `<prefix>.edges` exist. The two files should be binary, where

*   `<prefix>.nodes` contains a sequence of `(u32, u32)` pairs representing `(node_id, degree)`.
*   `<prefix>.edges` contains a sequence of `u32` values indicating the concatenation of edge destinations for all nodes indicated above.

The program is just going to map these two files into memory and read them, so you want to make sure that data have the appropriate endian-ness for your system.

The `hilbert` option requires a `<prefix>` for which `<prefix>.upper` and `<prefix>.lower` exist. The two files should be binary, where

*   `<prefix>.upper` contains a sequence of `((u16, u16), u32)` values, indicating a pair of upper 16 bits of node identifiers, and a count of how many edges have this pair of upper 16 bits.
*   `<prefix>.lower` contains a concatenated sequence of `(u16, u16)` values for the lower 16 bits for each edge in each group above.

The easiest way to get a feel for what these should look like is to invoke the `to_hilbert` option with `<prefix>` valid for data in the `vertex` layout, and it will print to the screen what the data look like laid out in Hilbert format. If you change the code to write the data to disk rather than to the terminal, you should be good to go (remember, `.upper` and `.lower`, not `.nodes` and `.edges`).

## Notes

There is a [companion COST repository](https://github.com/MicrosoftResearch/NaiadSamples) managed by Microsoft Research, including the state of the project several months ago. This may be helpful if you are interested in the corresponding C# implementations. The repository also contains [Naiad](http://research.microsoft.com/Naiad/) implementations that were done more recently. I am no longer affiliated with Microsoft and cannot commit to the repository (nor, historically, do they accept pull requests), and must apologize for the sorry state I left the code in. It may be cleaned up in the future (either by me, or other more industrious souls), but the code I presently used is contained in this repository.
