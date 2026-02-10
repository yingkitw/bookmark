use chrono::{DateTime, Utc};
use serde::Serialize;

use super::{EdgeType, KnowledgeGraph, NodeType};

/// Export graph to DOT format (Graphviz)
pub fn to_dot(graph: &KnowledgeGraph) -> String {
    let mut dot = String::from("digraph BookmarkKnowledgeGraph {\n");
    dot.push_str("    rankdir=LR;\n");
    dot.push_str("    node [shape=box];\n\n");

    for node in &graph.nodes {
        let (color, shape) = match node.node_type {
            NodeType::Bookmark => ("lightblue", "box"),
            NodeType::Domain => ("lightgreen", "ellipse"),
            NodeType::Folder => ("lightyellow", "folder"),
            NodeType::Tag => ("lightsalmon", "diamond"),
            NodeType::Category => ("plum", "octagon"),
        };
        dot.push_str(&format!(
            "    \"{}\" [label=\"{}\", fillcolor={}, style=filled, shape={}];\n",
            escape_dot_id(&node.id),
            escape_dot_label(&node.title),
            color,
            shape
        ));
    }

    dot.push_str("\n");

    for edge in &graph.edges {
        let style = match edge.edge_type {
            EdgeType::BelongsToDomain => "[color=blue, penwidth=2]",
            EdgeType::InFolder => "[color=green, penwidth=1]",
            EdgeType::SameDomain => "[color=gray, penwidth=0.5, style=dashed]",
            EdgeType::HasTag => "[color=orange, penwidth=1, style=dotted]",
            EdgeType::InCategory => "[color=purple, penwidth=1.5]",
            EdgeType::SimilarContent => "[color=red, penwidth=0.5, style=dashed]",
        };
        dot.push_str(&format!(
            "    \"{}\" -> \"{}\" {};\n",
            escape_dot_id(&edge.source),
            escape_dot_id(&edge.target),
            style
        ));
    }

    dot.push_str("}\n");
    dot
}

/// Export graph to JSON format
pub fn to_json(graph: &KnowledgeGraph) -> String {
    #[derive(Serialize)]
    struct JsonGraph {
        nodes: Vec<JsonNode>,
        edges: Vec<JsonEdge>,
        metadata: JsonMetadata,
    }

    #[derive(Serialize)]
    struct JsonNode {
        id: String,
        title: String,
        node_type: String,
        url: Option<String>,
        domain: Option<String>,
        folder: Option<String>,
        size: usize,
    }

    #[derive(Serialize)]
    struct JsonEdge {
        source: String,
        target: String,
        edge_type: String,
        weight: f64,
    }

    #[derive(Serialize)]
    struct JsonMetadata {
        total_nodes: usize,
        total_edges: usize,
        bookmark_count: usize,
        domain_count: usize,
        folder_count: usize,
        generated_at: DateTime<Utc>,
    }

    let json_nodes: Vec<JsonNode> = graph
        .nodes
        .iter()
        .map(|n| JsonNode {
            id: n.id.clone(),
            title: n.title.clone(),
            node_type: format!("{:?}", n.node_type).to_lowercase(),
            url: n.url.clone(),
            domain: n.domain.clone(),
            folder: n.folder.clone(),
            size: n.size,
        })
        .collect();

    let json_edges: Vec<JsonEdge> = graph
        .edges
        .iter()
        .map(|e| JsonEdge {
            source: e.source.clone(),
            target: e.target.clone(),
            edge_type: format!("{:?}", e.edge_type).to_lowercase(),
            weight: e.weight,
        })
        .collect();

    let json_graph = JsonGraph {
        nodes: json_nodes,
        edges: json_edges,
        metadata: JsonMetadata {
            total_nodes: graph.metadata.total_nodes,
            total_edges: graph.metadata.total_edges,
            bookmark_count: graph.metadata.bookmark_count,
            domain_count: graph.metadata.domain_count,
            folder_count: graph.metadata.folder_count,
            generated_at: graph.metadata.generated_at,
        },
    };

    serde_json::to_string_pretty(&json_graph).unwrap_or_default()
}

/// Export graph to GEXF format (Gephi)
pub fn to_gexf(graph: &KnowledgeGraph) -> String {
    let mut gexf = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<gexf xmlns="http://www.gexf.net/1.2draft" version="1.2">
    <graph mode="static" defaultedgetype="directed">
"#,
    );

    gexf.push_str(
        r#"        <attributes class="node">
            <attribute id="0" title="node_type" type="string"/>
            <attribute id="1" title="url" type="string"/>
            <attribute id="2" title="domain" type="string"/>
            <attribute id="3" title="folder" type="string"/>
        </attributes>
"#,
    );

    gexf.push_str("        <nodes>\n");
    for node in &graph.nodes {
        let node_type_str = format!("{:?}", node.node_type).to_lowercase();
        gexf.push_str(&format!(
            r#"            <node id="{}" label="{}">
                <attvalues>
                    <attvalue for="0" value="{}"/>"#,
            escape_xml(&node.id),
            escape_xml(&node.title),
            escape_xml(&node_type_str)
        ));

        if let Some(ref url) = node.url {
            gexf.push_str(&format!(
                r#"
                    <attvalue for="1" value="{}"/>"#,
                escape_xml(url)
            ));
        }

        if let Some(ref domain) = node.domain {
            gexf.push_str(&format!(
                r#"
                    <attvalue for="2" value="{}"/>"#,
                escape_xml(domain)
            ));
        }

        if let Some(ref folder) = node.folder {
            gexf.push_str(&format!(
                r#"
                    <attvalue for="3" value="{}"/>"#,
                escape_xml(folder)
            ));
        }

        gexf.push_str(
            r#"
                </attvalues>
            </node>"#,
        );
        gexf.push('\n');
    }
    gexf.push_str("        </nodes>\n");

    gexf.push_str("        <edges>\n");
    for (i, edge) in graph.edges.iter().enumerate() {
        let edge_type_str = format!("{:?}", edge.edge_type).to_lowercase();
        gexf.push_str(&format!(
            r#"            <edge id="{}" source="{}" target="{}" weight="{}" label="{}"/>"#,
            i,
            escape_xml(&edge.source),
            escape_xml(&edge.target),
            edge.weight,
            escape_xml(&edge_type_str)
        ));
        gexf.push('\n');
    }
    gexf.push_str("        </edges>\n");

    gexf.push_str("    </graph>\n</gexf>");
    gexf
}

/// Export graph to interactive HTML visualization using D3.js
pub fn to_html(graph: &KnowledgeGraph) -> String {
    let graph_json = to_json(graph);
    format!(
        "{preamble}{graph_json}{postamble}",
        preamble = HTML_PREAMBLE,
        graph_json = graph_json,
        postamble = HTML_POSTAMBLE
    )
}

/// Export graph to interactive HTML that loads data dynamically from JS file
pub fn to_html_dynamic(data_path: &std::path::Path) -> String {
    let data_filename = data_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("graph.data.js");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Bookmark Knowledge Graph</title>
{css}
</head>
<body class="dark">
<div id="loading">
  <div class="spinner"></div>
  <div>Loading graph data...</div>
</div>
{controls}
<svg id="graph"></svg>
<script src="https://d3js.org/d3.v7.min.js"></script>
<script src="{data_filename}"></script>
<script>
function initGraph() {{
  if (typeof window.graphData === 'undefined') {{
    document.getElementById('loading').innerHTML = '<div style="color:#ef5350">Error: Graph data not loaded. Make sure {data_filename} is in the same directory.</div>';
    return;
  }}
  document.getElementById('loading').style.display = 'none';
  const graphData = window.graphData;
  {d3_script}
}}
if (document.readyState === 'loading') {{
  document.addEventListener('DOMContentLoaded', initGraph);
}} else {{
  initGraph();
}}
</script>
</body>
</html>"#,
        css = HTML_CSS,
        controls = HTML_CONTROLS,
        data_filename = data_filename,
        d3_script = D3_GRAPH_SCRIPT,
    )
}

/// Export graph data as JavaScript file (for local file loading)
pub fn to_js_data(graph: &KnowledgeGraph) -> String {
    let json_content = to_json(graph);
    format!(
        "// Bookmark Knowledge Graph Data\n// Generated by bookmark tool\nwindow.graphData = {};\n",
        json_content
    )
}

// --- Escape helpers ---

fn escape_dot_id(s: &str) -> String {
    s.replace('"', "_")
        .replace('\\', "_")
        .replace(|c: char| c.is_whitespace(), "_")
}

fn escape_dot_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('|', "\\|")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('<', "\\<")
        .replace('>', "\\>")
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// --- HTML template constants ---

const HTML_CSS: &str = r#"<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; overflow: hidden; }
  body.dark { background: #1a1a2e; color: #e0e0e0; }
  body.light { background: #f5f5f5; color: #333; }
  #graph { width: 100vw; height: 100vh; }
  #loading {
    position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
    font-size: 18px; z-index: 100; text-align: center;
  }
  #controls {
    position: fixed; top: 16px; left: 16px; z-index: 10;
    padding: 16px; border-radius: 8px; min-width: 240px;
    font-size: 13px; backdrop-filter: blur(12px);
  }
  body.dark #controls { background: rgba(30,30,60,0.9); border: 1px solid #333; }
  body.light #controls { background: rgba(255,255,255,0.95); border: 1px solid #ddd; }
  #controls h3 { margin-bottom: 12px; font-size: 15px; }
  .ctrl-row { margin-bottom: 8px; display: flex; align-items: center; gap: 8px; }
  .ctrl-row label { min-width: 60px; }
  .ctrl-row input[type=range] { flex: 1; }
  .legend { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 10px; }
  .legend-item { display: flex; align-items: center; gap: 4px; font-size: 11px; }
  .legend-dot { width: 10px; height: 10px; border-radius: 50%; }
  #tooltip {
    position: fixed; padding: 8px 12px; border-radius: 6px; font-size: 12px;
    pointer-events: none; display: none; max-width: 320px; z-index: 20;
  }
  body.dark #tooltip { background: rgba(0,0,0,0.85); color: #fff; }
  body.light #tooltip { background: rgba(255,255,255,0.95); color: #333; border: 1px solid #ccc; }
  #stats {
    position: fixed; bottom: 16px; left: 16px; font-size: 11px; opacity: 0.7;
  }
  button.theme-btn {
    position: fixed; top: 16px; right: 16px; z-index: 10;
    padding: 6px 14px; border-radius: 6px; cursor: pointer; border: 1px solid #555;
    font-size: 12px;
  }
  body.dark button.theme-btn { background: #333; color: #eee; }
  body.light button.theme-btn { background: #fff; color: #333; }
  .filter-group { margin-top: 8px; }
  .filter-group label { font-size: 11px; cursor: pointer; }
  .filter-group input { margin-right: 4px; }
  .spinner {
    border: 3px solid #f3f3f3; border-top: 3px solid #4fc3f7;
    border-radius: 50%; width: 40px; height: 40px;
    animation: spin 1s linear infinite; margin: 0 auto 16px;
  }
  @keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
</style>"#;

const HTML_CONTROLS: &str = r#"<div id="controls">
  <h3>Knowledge Graph</h3>
  <div class="ctrl-row"><label>Charge</label><input type="range" id="charge" min="-500" max="-10" value="-120"></div>
  <div class="ctrl-row"><label>Distance</label><input type="range" id="distance" min="20" max="300" value="80"></div>
  <div class="filter-group">
    <div><label><input type="checkbox" data-type="bookmark" checked> Bookmarks</label></div>
    <div><label><input type="checkbox" data-type="domain" checked> Domains</label></div>
    <div><label><input type="checkbox" data-type="folder" checked> Folders</label></div>
    <div><label><input type="checkbox" data-type="tag" checked> Tags</label></div>
    <div><label><input type="checkbox" data-type="category" checked> Categories</label></div>
  </div>
  <div class="legend">
    <div class="legend-item"><div class="legend-dot" style="background:#4fc3f7"></div>Bookmark</div>
    <div class="legend-item"><div class="legend-dot" style="background:#81c784"></div>Domain</div>
    <div class="legend-item"><div class="legend-dot" style="background:#fff176"></div>Folder</div>
    <div class="legend-item"><div class="legend-dot" style="background:#ff8a65"></div>Tag</div>
    <div class="legend-item"><div class="legend-dot" style="background:#ce93d8"></div>Category</div>
  </div>
</div>
<button class="theme-btn" onclick="toggleTheme()">Toggle Theme</button>
<div id="tooltip"></div>
<div id="stats"></div>"#;

const D3_GRAPH_SCRIPT: &str = r#"const colorMap = { bookmark:'#4fc3f7', domain:'#81c784', folder:'#fff176', tag:'#ff8a65', category:'#ce93d8' };
const radiusMap = { bookmark:5, domain:10, folder:8, tag:7, category:12 };

let visibleTypes = new Set(['bookmark','domain','folder','tag','category']);
const svg = d3.select('#graph');
const width = window.innerWidth, height = window.innerHeight;
svg.attr('width', width).attr('height', height);

const g = svg.append('g');
svg.call(d3.zoom().scaleExtent([0.1, 8]).on('zoom', (e) => g.attr('transform', e.transform)));

let simulation, linkSel, nodeSel, labelSel;

function filterData() {
  const nodes = graphData.nodes.filter(n => visibleTypes.has(n.node_type));
  const nodeIds = new Set(nodes.map(n => n.id));
  const edges = graphData.edges.filter(e => {
    const sourceId = typeof e.source === 'object' ? e.source.id : e.source;
    const targetId = typeof e.target === 'object' ? e.target.id : e.target;
    return nodeIds.has(sourceId) && nodeIds.has(targetId);
  });
  return { nodes, edges };
}

function render() {
  if (graphData.nodes.length === 0) return;
  const data = filterData();
  g.selectAll('*').remove();

  const edgeColorMap = {
    belongstodomain:'#42a5f5', infolder:'#66bb6a', samedomain:'#78909c',
    hastag:'#ffa726', incategory:'#ab47bc', similarcontent:'#ef5350'
  };

  linkSel = g.append('g').selectAll('line').data(data.edges).join('line')
    .attr('stroke', d => edgeColorMap[d.edge_type] || '#555')
    .attr('stroke-opacity', 0.4)
    .attr('stroke-width', d => Math.max(0.5, d.weight * 2));

  nodeSel = g.append('g').selectAll('circle').data(data.nodes).join('circle')
    .attr('r', d => Math.max(radiusMap[d.node_type] || 5, Math.sqrt(d.size) * 3))
    .attr('fill', d => colorMap[d.node_type] || '#999')
    .attr('stroke', '#fff').attr('stroke-width', 0.5)
    .style('cursor', 'pointer')
    .call(d3.drag().on('start', dragStart).on('drag', dragging).on('end', dragEnd))
    .on('mouseover', showTooltip).on('mouseout', hideTooltip)
    .on('click', (e, d) => { if (d.url) window.open(d.url, '_blank'); });

  labelSel = g.append('g').selectAll('text').data(data.nodes.filter(n => n.node_type !== 'bookmark')).join('text')
    .text(d => d.title.length > 20 ? d.title.slice(0, 20) + '...' : d.title)
    .attr('font-size', 9).attr('dx', 12).attr('dy', 3)
    .attr('fill', document.body.classList.contains('dark') ? '#ccc' : '#555');

  simulation = d3.forceSimulation(data.nodes)
    .force('link', d3.forceLink(data.edges).id(d => d.id).distance(+document.getElementById('distance').value))
    .force('charge', d3.forceManyBody().strength(+document.getElementById('charge').value))
    .force('center', d3.forceCenter(width / 2, height / 2))
    .force('collision', d3.forceCollide().radius(d => (radiusMap[d.node_type] || 5) + 2))
    .on('tick', () => {
      linkSel.attr('x1', d => d.source.x).attr('y1', d => d.source.y)
             .attr('x2', d => d.target.x).attr('y2', d => d.target.y);
      nodeSel.attr('cx', d => d.x).attr('cy', d => d.y);
      labelSel.attr('x', d => d.x).attr('y', d => d.y);
    });

  document.getElementById('stats').textContent =
    `Nodes: ${data.nodes.length} | Edges: ${data.edges.length} | Bookmarks: ${graphData.metadata.bookmark_count} | Domains: ${graphData.metadata.domain_count}`;
}

function showTooltip(e, d) {
  const tip = document.getElementById('tooltip');
  let html = `<strong>${d.title}</strong><br>Type: ${d.node_type}`;
  if (d.url) html += `<br>URL: ${d.url}`;
  if (d.domain) html += `<br>Domain: ${d.domain}`;
  if (d.folder) html += `<br>Folder: ${d.folder}`;
  html += `<br>Size: ${d.size}`;
  tip.innerHTML = html;
  tip.style.display = 'block';
  tip.style.left = (e.clientX + 12) + 'px';
  tip.style.top = (e.clientY - 12) + 'px';
}
function hideTooltip() { document.getElementById('tooltip').style.display = 'none'; }

function dragStart(e, d) { if (!e.active) simulation.alphaTarget(0.3).restart(); d.fx = d.x; d.fy = d.y; }
function dragging(e, d) { d.fx = e.x; d.fy = e.y; }
function dragEnd(e, d) { if (!e.active) simulation.alphaTarget(0); d.fx = null; d.fy = null; }

function toggleTheme() {
  document.body.classList.toggle('dark');
  document.body.classList.toggle('light');
  if (labelSel) {
    labelSel.attr('fill', document.body.classList.contains('dark') ? '#ccc' : '#555');
  }
}

document.getElementById('charge').addEventListener('input', () => {
  if (simulation) simulation.force('charge', d3.forceManyBody().strength(+document.getElementById('charge').value)).alpha(0.3).restart();
});
document.getElementById('distance').addEventListener('input', () => {
  if (simulation) { simulation.force('link').distance(+document.getElementById('distance').value); simulation.alpha(0.3).restart(); }
});
document.querySelectorAll('.filter-group input').forEach(cb => {
  cb.addEventListener('change', () => {
    if (cb.checked) visibleTypes.add(cb.dataset.type); else visibleTypes.delete(cb.dataset.type);
    render();
  });
});

render();"#;

const HTML_PREAMBLE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Bookmark Knowledge Graph</title>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; overflow: hidden; }
  body.dark { background: #1a1a2e; color: #e0e0e0; }
  body.light { background: #f5f5f5; color: #333; }
  #graph { width: 100vw; height: 100vh; }
  #controls {
    position: fixed; top: 16px; left: 16px; z-index: 10;
    padding: 16px; border-radius: 8px; min-width: 240px;
    font-size: 13px; backdrop-filter: blur(12px);
  }
  body.dark #controls { background: rgba(30,30,60,0.9); border: 1px solid #333; }
  body.light #controls { background: rgba(255,255,255,0.95); border: 1px solid #ddd; }
  #controls h3 { margin-bottom: 12px; font-size: 15px; }
  .ctrl-row { margin-bottom: 8px; display: flex; align-items: center; gap: 8px; }
  .ctrl-row label { min-width: 60px; }
  .ctrl-row input[type=range] { flex: 1; }
  .legend { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 10px; }
  .legend-item { display: flex; align-items: center; gap: 4px; font-size: 11px; }
  .legend-dot { width: 10px; height: 10px; border-radius: 50%; }
  #tooltip {
    position: fixed; padding: 8px 12px; border-radius: 6px; font-size: 12px;
    pointer-events: none; display: none; max-width: 320px; z-index: 20;
  }
  body.dark #tooltip { background: rgba(0,0,0,0.85); color: #fff; }
  body.light #tooltip { background: rgba(255,255,255,0.95); color: #333; border: 1px solid #ccc; }
  #stats {
    position: fixed; bottom: 16px; left: 16px; font-size: 11px; opacity: 0.7;
  }
  button.theme-btn {
    position: fixed; top: 16px; right: 16px; z-index: 10;
    padding: 6px 14px; border-radius: 6px; cursor: pointer; border: 1px solid #555;
    font-size: 12px;
  }
  body.dark button.theme-btn { background: #333; color: #eee; }
  body.light button.theme-btn { background: #fff; color: #333; }
  .filter-group { margin-top: 8px; }
  .filter-group label { font-size: 11px; cursor: pointer; }
  .filter-group input { margin-right: 4px; }
</style>
</head>
<body class="dark">
<div id="controls">
  <h3>Knowledge Graph</h3>
  <div class="ctrl-row"><label>Charge</label><input type="range" id="charge" min="-500" max="-10" value="-120"></div>
  <div class="ctrl-row"><label>Distance</label><input type="range" id="distance" min="20" max="300" value="80"></div>
  <div class="filter-group">
    <div><label><input type="checkbox" data-type="bookmark" checked> Bookmarks</label></div>
    <div><label><input type="checkbox" data-type="domain" checked> Domains</label></div>
    <div><label><input type="checkbox" data-type="folder" checked> Folders</label></div>
    <div><label><input type="checkbox" data-type="tag" checked> Tags</label></div>
    <div><label><input type="checkbox" data-type="category" checked> Categories</label></div>
  </div>
  <div class="legend">
    <div class="legend-item"><div class="legend-dot" style="background:#4fc3f7"></div>Bookmark</div>
    <div class="legend-item"><div class="legend-dot" style="background:#81c784"></div>Domain</div>
    <div class="legend-item"><div class="legend-dot" style="background:#fff176"></div>Folder</div>
    <div class="legend-item"><div class="legend-dot" style="background:#ff8a65"></div>Tag</div>
    <div class="legend-item"><div class="legend-dot" style="background:#ce93d8"></div>Category</div>
  </div>
</div>
<button class="theme-btn" onclick="toggleTheme()">Toggle Theme</button>
<div id="tooltip"></div>
<div id="stats"></div>
<svg id="graph"></svg>
<script src="https://d3js.org/d3.v7.min.js"></script>
<script>
const graphData = "#;

const HTML_POSTAMBLE: &str = r#";
const colorMap = { bookmark:'#4fc3f7', domain:'#81c784', folder:'#fff176', tag:'#ff8a65', category:'#ce93d8' };
const radiusMap = { bookmark:5, domain:10, folder:8, tag:7, category:12 };

let visibleTypes = new Set(['bookmark','domain','folder','tag','category']);
const svg = d3.select('#graph');
const width = window.innerWidth, height = window.innerHeight;
svg.attr('width', width).attr('height', height);

const g = svg.append('g');
svg.call(d3.zoom().scaleExtent([0.1, 8]).on('zoom', (e) => g.attr('transform', e.transform)));

let simulation, linkSel, nodeSel, labelSel;

function filterData() {
  const nodes = graphData.nodes.filter(n => visibleTypes.has(n.node_type));
  const nodeIds = new Set(nodes.map(n => n.id));
  const edges = graphData.edges.filter(e => {
    const sourceId = typeof e.source === 'object' ? e.source.id : e.source;
    const targetId = typeof e.target === 'object' ? e.target.id : e.target;
    return nodeIds.has(sourceId) && nodeIds.has(targetId);
  });
  return { nodes, edges };
}

function render() {
  if (graphData.nodes.length === 0) return;
  const data = filterData();
  g.selectAll('*').remove();

  const edgeColorMap = {
    belongstodomain:'#42a5f5', infolder:'#66bb6a', samedomain:'#78909c',
    hastag:'#ffa726', incategory:'#ab47bc', similarcontent:'#ef5350'
  };

  linkSel = g.append('g').selectAll('line').data(data.edges).join('line')
    .attr('stroke', d => edgeColorMap[d.edge_type] || '#555')
    .attr('stroke-opacity', 0.4)
    .attr('stroke-width', d => Math.max(0.5, d.weight * 2));

  nodeSel = g.append('g').selectAll('circle').data(data.nodes).join('circle')
    .attr('r', d => Math.max(radiusMap[d.node_type] || 5, Math.sqrt(d.size) * 3))
    .attr('fill', d => colorMap[d.node_type] || '#999')
    .attr('stroke', '#fff').attr('stroke-width', 0.5)
    .style('cursor', 'pointer')
    .call(d3.drag().on('start', dragStart).on('drag', dragging).on('end', dragEnd))
    .on('mouseover', showTooltip).on('mouseout', hideTooltip)
    .on('click', (e, d) => { if (d.url) window.open(d.url, '_blank'); });

  labelSel = g.append('g').selectAll('text').data(data.nodes.filter(n => n.node_type !== 'bookmark')).join('text')
    .text(d => d.title.length > 20 ? d.title.slice(0, 20) + '...' : d.title)
    .attr('font-size', 9).attr('dx', 12).attr('dy', 3)
    .attr('fill', document.body.classList.contains('dark') ? '#ccc' : '#555');

  simulation = d3.forceSimulation(data.nodes)
    .force('link', d3.forceLink(data.edges).id(d => d.id).distance(+document.getElementById('distance').value))
    .force('charge', d3.forceManyBody().strength(+document.getElementById('charge').value))
    .force('center', d3.forceCenter(width / 2, height / 2))
    .force('collision', d3.forceCollide().radius(d => (radiusMap[d.node_type] || 5) + 2))
    .on('tick', () => {
      linkSel.attr('x1', d => d.source.x).attr('y1', d => d.source.y)
             .attr('x2', d => d.target.x).attr('y2', d => d.target.y);
      nodeSel.attr('cx', d => d.x).attr('cy', d => d.y);
      labelSel.attr('x', d => d.x).attr('y', d => d.y);
    });

  document.getElementById('stats').textContent =
    `Nodes: ${data.nodes.length} | Edges: ${data.edges.length} | Bookmarks: ${graphData.metadata.bookmark_count} | Domains: ${graphData.metadata.domain_count}`;
}

function showTooltip(e, d) {
  const tip = document.getElementById('tooltip');
  let html = `<strong>${d.title}</strong><br>Type: ${d.node_type}`;
  if (d.url) html += `<br>URL: ${d.url}`;
  if (d.domain) html += `<br>Domain: ${d.domain}`;
  if (d.folder) html += `<br>Folder: ${d.folder}`;
  html += `<br>Size: ${d.size}`;
  tip.innerHTML = html;
  tip.style.display = 'block';
  tip.style.left = (e.clientX + 12) + 'px';
  tip.style.top = (e.clientY - 12) + 'px';
}
function hideTooltip() { document.getElementById('tooltip').style.display = 'none'; }

function dragStart(e, d) { if (!e.active) simulation.alphaTarget(0.3).restart(); d.fx = d.x; d.fy = d.y; }
function dragging(e, d) { d.fx = e.x; d.fy = e.y; }
function dragEnd(e, d) { if (!e.active) simulation.alphaTarget(0); d.fx = null; d.fy = null; }

function toggleTheme() {
  document.body.classList.toggle('dark');
  document.body.classList.toggle('light');
  if (labelSel) {
    labelSel.attr('fill', document.body.classList.contains('dark') ? '#ccc' : '#555');
  }
}

document.getElementById('charge').addEventListener('input', () => {
  if (simulation) simulation.force('charge', d3.forceManyBody().strength(+document.getElementById('charge').value)).alpha(0.3).restart();
});
document.getElementById('distance').addEventListener('input', () => {
  if (simulation) { simulation.force('link').distance(+document.getElementById('distance').value); simulation.alpha(0.3).restart(); }
});
document.querySelectorAll('.filter-group input').forEach(cb => {
  cb.addEventListener('change', () => {
    if (cb.checked) visibleTypes.add(cb.dataset.type); else visibleTypes.delete(cb.dataset.type);
    render();
  });
});

render();
</script>
</body>
</html>"#;
