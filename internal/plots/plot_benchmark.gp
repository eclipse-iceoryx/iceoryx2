#!/usr/bin/gnuplot -p

set style line 101 lc rgb '#808080' lt 1 lw 1
set border 3 front ls 101
set tics nomirror out scale 0.75

set style line 102 lc rgb '#d6d7d9' lt 0 lw 1
set grid back ls 102

set xlabel "payload size in kb"
set ylabel "latency in us"
set title "benchmark"
set logscale x 2
set logscale y 2
set xrange [0:4096]
set yrange [0:3100]
set key left
set term png transparent truecolor size 1200,300
set output 'benchmark.png'

set style line 1 \
    linecolor rgb '#97de26' \
    linetype 1 linewidth 4 \
    pointtype 0 pointsize 1
set style line 2 \
    linecolor rgb '#2697de' \
    linetype 1 linewidth 2 \
    pointtype 0 pointsize 1
set style line 3 \
    linecolor rgb '#282828' \
    linetype 0 linewidth 2 \
    pointtype 0 pointsize 1
set style line 4 \
    linecolor rgb '#282828' \
    linetype 5 linewidth 2 \
    pointtype 0 pointsize 1

plot 'benchmark_results.dat' index 0 with linespoints linestyle 1 title "iceoryx2", \
     ''                      index 1 with linespoints linestyle 2 title "iceoryx", \
     ''                      index 2 with linespoints linestyle 3 title "message queue", \
     ''                      index 3 with linespoints linestyle 4 title "unix domain socket" \
