"use strict";
(function () {
  const data = window.BENCHMARK_DATA;
  // Render footer
  document.getElementById("download-button").onclick = () => {
    const dataUrl = "data:," + JSON.stringify(data, null, 2);
    const a = document.createElement("a");
    a.href = dataUrl;
    a.download = "benchmark_data.json";
    a.click();
  };

  const benches = _(data["entries"]).values().first();
  const colorway = ["#ff7f0e", "#2ca02c", "#1f77b4"];

  let cmpChartData = _(_.last(benches)["benches"])
    .map((bench) => {
      let [_, impl, sample_size] = bench.name.split("_");
      bench.impl = impl;
      bench.sample_size = sample_size;
      return bench;
    })
    .orderBy("value")
    .groupBy("impl")
    .map((group) => ({
      x: _.map(group, "sample_size"),
      y: _.map(group, (x) => x.value / 1e9), // convert from ns to sec
      name: group[0].impl,
      type: "bar",
    }))
    .value();

  var cmpLayout = {
    title: "PyXIRR vs other implementations",
    barmode: "group",
    xaxis: {
      title: "Sample size",
      type: "category",
    },
    yaxis: {
      title: "Execution time",
      rangemode: "tozero",
      autorange: true,
      tickformat: "0.2s",
      ticksuffix: "s",
      hoverformat: ".4s",
    },
    colorway: colorway,
  };

  Plotly.newPlot("comparison", cmpChartData, cmpLayout);

  let perfChartData = [
    {
      y: _(benches)
        .map("benches")
        .flatten()
        .filter({ name: "bench_rust_100" })
        .map((x) => x.value / 1e9)
        .value(),
      x: _.range(benches.length),
      text: _.map(benches,(x) => x.commit.id.slice(0, 7))
    },
  ];

  var perfLayout = {
    title: "PyXIRR performance over time",
    barmode: "group",
    xaxis: {
      title: "Commit",
      ticktext: perfChartData[0].text,
      tickvals: perfChartData[0].x
    },
    yaxis: {
      title: "Execution time",
      rangemode: "tozero",
      autorange: true,
      tickformat: ".2s",
      hoverformat: ".4s",
      ticksuffix: "s",
    },
    colorway: colorway,
  };

  Plotly.newPlot("performance", perfChartData, perfLayout);
})();
