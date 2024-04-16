#!/usr/bin/gnuplot -p

set style line 101 lc rgb '#000000' lt 1 lw 1
set border 3 front ls 101
set tics nomirror out scale 0.75

set style line 102 lc rgb '#d6d7d9' lt 0 lw 1
set grid back ls 102

set xtic rotate by 90 scale 0
set ytics rotate by 90
set ylabel "iceoryx2 Latency On Different Platforms"
set y2label 'latency in nanoseconds (less is better)' offset 2.5
set xlabel ' '
set size 0.6, 1
set xtics left offset 0,-7 font ", 8"
set bmargin 8
unset key

set term png transparent truecolor size 700,1200
set output 'benchmark_architecture.png'

set boxwidth 0.6
set style fill solid noborder
set yrange[0:1000]

plot 'benchmark_architecture_os_comparision.dat' using 0:3:xtic(sprintf("%s\n    %s", stringcolumn(1), stringcolumn(2))) with boxes lc rgb "#738f4d"

system("convert -rotate 90 benchmark_architecture.png benchmark_architecture.png")
system("convert benchmark_architecture.png -trim +repage benchmark_architecture.png")
