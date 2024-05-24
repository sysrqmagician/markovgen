flamegraphs:
    #!/bin/sh
    CARGO_PROFILE_RELEASE_PANIC=abort CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin markovcli --flamechart -o flamegraph_compile.svg -f cli_no_print -- compile benches/US_Census_1990_Frequent_Male_First_Names.txt flamegraph.graph.bin
    CARGO_PROFILE_RELEASE_PANIC=abort CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin markovcli --flamechart -o flamegraph_sample.svg -f cli_no_print -- sample flamegraph.graph.bin 10000000
