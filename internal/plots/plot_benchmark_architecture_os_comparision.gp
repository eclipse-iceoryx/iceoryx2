#!/usr/bin/gnuplot -p

set xtic rotate by 90 scale 0
set ytics rotate by 90
set ylabel "iceoryx2 Latency On Different Platforms"
set y2label 'Latency in us (less is better)' offset 2.5
set xlabel ' '
set size 0.6, 1
set xtics left offset 0,-6 font ", 7"
set bmargin 7
unset key

set term png transparent truecolor size 700,1200
set output 'benchmark_architecture.png'

set boxwidth 0.6
set style fill solid noborder
set yrange[0:1000]

plot 'benchmark_architecture_os_comparision.dat' using 0:3:xtic(sprintf("%s\n    %s", stringcolumn(1), stringcolumn(2))) with boxes lc rgb "#738f4d"

system("convert -rotate 90 benchmark_architecture.png benchmark_architecture.png")
system("convert benchmark_architecture.png -trim +repage benchmark_architecture.png")
