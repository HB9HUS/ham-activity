/* -------------------------------------------------------------
   Helper: read a query‑string parameter (returns null if absent)
   ------------------------------------------------------------- */
function getQueryParam(name) {
  const params = new URLSearchParams(window.location.search);
  return params.get(name);
}

/* -------------------------------------------------------------
   Helper: hide loading elements
   ------------------------------------------------------------- */
function hideLoading() {
  const loadingEl = document.querySelector('.loading');
  if (loadingEl) loadingEl.remove();   // removes it from the DOM
}


/* -------------------------------------------------------------
   Render helpers
   ------------------------------------------------------------- */
function makeList(arr, urlBase = null) {
  const ul = document.createElement('ul');
  ul.className = 'pill-list';

  (arr || []).forEach(item => {
    const li = document.createElement('li');
    li.className = 'list-group-item p-1';

    if (urlBase) {
      // Build the full href – encode the item to keep URLs safe
      const a = document.createElement('a');
      a.href = urlBase + encodeURIComponent(item);
      a.textContent = item;
      a.target = '_blank';          // open in a new tab (optional)
      a.rel = 'noopener noreferrer';// security best practice
      li.appendChild(a);
    } else {
      li.textContent = item;
    }

    ul.appendChild(li);
  });

  return ul;
}
/* -------------------------------------------------------------
   Main async entry point
   ------------------------------------------------------------- */
(async () => {
  const container = document.getElementById('content');

  // ---- Determine region (fallback to CQ-14) ----
  const region = getQueryParam('region') || 'CQ_14';
  const apiUrl = `http://localhost:8000/region/${encodeURIComponent(region)}`;

  try {
    const resp = await fetch(apiUrl);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = await resp.json();

    const qrz_com_url ='https://qrz.com/db/'

    hideLoading();

    /* ----- Region name ----- */
    const nameCard = document.createElement('div');
    nameCard.className = 'card mb-3';
    nameCard.innerHTML = `
      <div class="card-header fw-bold">Region</div>
      <div class="card-body"><p class="card-text">${data.name || '‑'}</p></div>`;
    container.appendChild(nameCard);

    /* ----- Spotters ----- */
    const spotCard = document.createElement('div');
    spotCard.className = 'card mb-3';
    spotCard.innerHTML = `
      <div class="card-header fw-bold">Spotters (${data.spotters?.length ?? 0})</div>
      <div class="card-body"></div>`;
    spotCard.querySelector('.card-body').appendChild(makeList(data.spotters, qrz_com_url), qrz_com_url);
    container.appendChild(spotCard);

    /* ----- Number of spots ----- */
    const numCard = document.createElement('div');
    numCard.className = 'card mb-3';
    numCard.innerHTML = `
      <div class="card-header fw-bold">Number of Spotter Spots</div>
      <div class="card-body"><p class="card-text">${data.num_spotter_spots ?? '‑'}</p></div>`;
    container.appendChild(numCard);

    /* ----- Band activities matrix (Bootstrap table) ----- */
    const bandCard = document.createElement('div');
    bandCard.className = 'card mb-3';
    bandCard.innerHTML = '<div class="card-header fw-bold">Band Activities</div>';
    const table = document.createElement('table');
    table.className = 'table table-sm table-striped text-center align-middle';
    const thead = document.createElement('thead');
    const headTr = document.createElement('tr');

    // top‑left empty cell
    const emptyTh = document.createElement('th');
    emptyTh.scope = 'col';
    headTr.appendChild(emptyTh);

    // band headers
    data.band_activities.forEach(b => {
      const th = document.createElement('th');
      th.scope = 'col';
      th.textContent = b.band;
      headTr.appendChild(th);
    });
    thead.appendChild(headTr);
    table.appendChild(thead);

    const tbody = document.createElement('tbody');

    const addRow = (label, accessor) => {
      const tr = document.createElement('tr');
      const th = document.createElement('th');
      th.scope = 'row';
      th.textContent = label;
      tr.appendChild(th);
      data.band_activities.forEach(b => {
        const td = document.createElement('td');
        td.appendChild(makeList(accessor(b), qrz_com_url));
        tr.appendChild(td);
      });
      tbody.appendChild(tr);
    };

    addRow('1 min',  b => b.active_1min);
    addRow('5 min',  b => b.active_5min);
    addRow('15 min', b => b.active_15min);

    table.appendChild(tbody);
    bandCard.appendChild(table);
    container.appendChild(bandCard);

  } catch (e) {
    container.innerHTML = `<div class="alert alert-danger" role="alert">
      Failed to load data for region “${region}”: ${e.message}
    </div>`;
  }
})();
