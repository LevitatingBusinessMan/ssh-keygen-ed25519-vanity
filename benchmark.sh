#!/bin/bash

word="sting"
threads=4

for i in {1..5}
do
	results=(0 0 0 0)
	for j in {1..4}
	do
		results[$j]="$(/usr/bin/time -f "%e" ./target/release/vanity ${word:0:$i} $threads 2>&1 >/dev/null | tail -n1)"
	done
	echo -n "${word:0:$i}: "
	echo "scale=3; (${results[0]} + ${results[1]} + ${results[2]}+ ${results[3]}) / 4" | bc -l
done
