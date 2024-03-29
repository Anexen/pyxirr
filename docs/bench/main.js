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
  const implCodes = { rust: 1, scipy: 2, python: 3 };

  const lastBenches = _.last(benches)["benches"].map((bench) => {
    let [_, impl, sample_size] = bench.name.split("_");
    bench.impl = impl;
    bench.sample_size = sample_size;
    bench.implCode = implCodes[impl];
    return bench;
  });

  const formatRatio = (x) => {
    return "x" + Plotly.d3.format("0.1f")(x);
  };
  const formatDuration = (x) => {
    return Plotly.d3.format("0.4s")(x.value / 1e9) + "s";
  };

  const cmpChartData = _(lastBenches)
    .orderBy(["implCode", "value"])
    .groupBy("impl")
    .map((impl) => ({
      x: _.map(impl, "sample_size"),
      y: _.map(impl, (x) => x.value / 1e9), // convert from ns to sec
      name: impl[0].impl,
      type: "bar",
      text: _.map(impl, (x, i) => {
        let duration = formatDuration(x);

        if (x.impl != "rust") {
          const group = _.filter(lastBenches, {
            impl: "rust",
            sample_size: x.sample_size,
          });
          const ratio = formatRatio(x.value / group[0].value);
          duration += `<br>(${ratio})`;
        }
        return duration;
      }),
      textposition: "auto",
      hoverinfo: "none",
    }))
    .value();

  const cmpLayout = {
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

  const compiled = _.template(`
    <tr>
        <th>Implementation</th>
        <th>Sample size</th>
        <th>Execution time</th>
    </tr>
    <% _.forEach(benches, function(bench) { %>
    <tr>
        <td><%- bench.impl %></td>
        <td><%- bench.sample_size %></td>
        <td><%- format(bench) %></td>
    </tr>
    <% }); %>
  `);

  document.getElementById("comparison-table").innerHTML = compiled({
    benches: _.orderBy(lastBenches, ["sample_size", "implCode"]),
    format: formatDuration,
  });

  const perfChartData = [
    {
      y: _(benches)
        .map("benches")
        .flatten()
        .filter({ name: "bench_rust_100" })
        .map((x) => x.value / 1e9)
        .value(),
      x: _.range(benches.length),
      text: _.map(benches, (x) => x.commit.id.slice(0, 7)),
    },
  ];

  const perfLayout = {
    title: "PyXIRR performance over time",
    barmode: "group",
    xaxis: {
      title: "Commit",
      ticktext: perfChartData[0].text,
      tickvals: perfChartData[0].x,
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
