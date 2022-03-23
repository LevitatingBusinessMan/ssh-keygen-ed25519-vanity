#!/bin/bash
for (( i=1; ; i++))
do
	echo -n "$i: "
	./target/release/vanity "$1" "$2" /tmp/$i 2>/dev/null
done
