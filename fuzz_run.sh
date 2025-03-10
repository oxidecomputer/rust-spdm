#!/bin/bash

pkill screen

if [ ! -d "fuzz-target/out" ];then
    mkdir fuzz-target/out
fi

for i in fuzz-target/out/*;do

    if [[ ! -f $i/default/crashes ]];then
        break
    fi

    if [[ "`ls -A $i/default/crashes`" != "" ]];then
        echo -e "\033[31m There are some crashes \033[0m"
        echo -e "\033[31m Path in fuzz-target/out/$i/default/crashes \033[0m"
        exit
    fi
done

if [ "core" != `cat /proc/sys/kernel/core_pattern` ];then
    if [ `id -u` -ne 0 ];then
        expect switch_root_run_cmd.sh
    else
        echo core >/proc/sys/kernel/core_pattern
        pushd /sys/devices/system/cpu
        echo performance | tee cpu*/cpufreq/scaling_governor
        popd
    fi
fi

rm -rf fuzz-target/out/*
cmds=(
"version_rsp"
"capability_rsp"
"algorithm_rsp"
"digest_rsp"
"certificate_rsp"
"challenge_rsp"
"measurement_rsp"
"keyexchange_rsp"
"pskexchange_rsp"
"finish_rsp"
"psk_finish_rsp"
"heartbeat_rsp"
"key_update_rsp"
"end_session_rsp"

"version_req"
"capability_req"
"algorithm_req"
"digest_req"
"certificate_req"
"challenge_req"
"measurement_req"
"key_exchange_req"
"psk_exchange_req"
"finish_req"
"psk_finish_req"
"heartbeat_req"
"key_update_req"
"end_session_req"
)

buildpackage=''
for i in ${cmds[@]};do
    buildpackage="-p $i $buildpackage";
done

echo "cargo afl build --features fuzz $buildpackage"


if [[ $1 = "Scoverage" ]]; then
    echo "$1"
    export RUSTFLAGS="-Zinstrument-coverage"
    export LLVM_PROFILE_FILE='fuzz_run%m.profraw'
fi

if [[ $1 = "Gcoverage" ]]; then
    echo "$1"
    export CARGO_INCREMENTAL=0
    export RUSTDOCFLAGS="-Cpanic=abort"
    export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
fi

cargo afl build --features fuzz $buildpackage

for ((i=0;i<${#cmds[*]};i++))
do
    echo ${cmds[$i]}
    screen -ls | grep ${cmds[$i]}
    if [[ $? -ne 0 ]]
    then
    screen -dmS ${cmds[$i]}
    fi
    screen -x -S ${cmds[$i]} -p 0 -X stuff "cargo afl fuzz -i fuzz-target/in/${cmds[$i]} -o fuzz-target/out/${cmds[$i]} target/debug/${cmds[$i]}"
    screen -x -S ${cmds[$i]} -p 0 -X stuff $'\n'
    sleep 3600
    screen -S ${cmds[$i]} -X quit
    sleep 5
done

if [[ $1 = "Scoverage" || $1 = "Gcoverage" ]]; then
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/fuzz_coverage/
    unset RUSTFLAGS
    unset LLVM_PROFILE_FILE
    unset CARGO_INCREMENTAL
    unset RUSTDOCFLAGS
    unset RUSTFLAGS
fi
