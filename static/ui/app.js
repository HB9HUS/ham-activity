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
function makeList(arr, urlBase = null, callInfoMap = null) {
  const ul = document.createElement('ul');
  ul.className = 'pill-list';

  (arr || []).forEach(item => {
    const li = document.createElement('li');
    li.className = 'list-group-item p-1';

    // ---------- Build the visual content (link or plain text) ----------
    if (urlBase) {
      const a = document.createElement('a');
      a.href = urlBase + encodeURIComponent(item);
      a.textContent = item;
      a.target = '_blank';
      a.rel = 'noopener noreferrer';
      li.appendChild(a);
    } else {
      li.textContent = item;
    }

    // ---------- Add mouse‑over tooltip if we have call‑info ----------
    if (callInfoMap && Object.prototype.hasOwnProperty.call(callInfoMap, item)) {
      const info = callInfoMap[item];   // { frequencies: [...], wpm: [...], db: [...] }

      // Helper to turn an array into a comma‑separated string (or “‑” if empty)
      const fmt = arr => (Array.isArray(arr) && arr.length) ? arr.join(', ') : '‑';

      // Build a multi‑line tooltip – the newline characters are respected
      // by most browsers when the `title` attribute is used.
      const tooltip = [
        `Freq: ${fmt(info.frequencies)} kHz`,
        `WPM:        ${fmt(info.wpm)}`,
        `dB:         ${fmt(info.db)}`
      ].join('\n');

      // Attach the tooltip to the <li> (or to the <a> if you prefer)
      li.title = tooltip;
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
  const {protocol, hostname, port} = window.location;
  const apiUrl = `${protocol}//${hostname}${port ? ':' + port : ''}/region/${encodeURIComponent(region)}`;

  let data;
  try {
    const resp = await fetch(apiUrl);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    data = await resp.json();
  } catch (e) {
    container.innerHTML = `<div class="alert alert-danger" role="alert">
      Failed to load data for region “${region}”: ${e.message}
    </div>`;
    return;
  }
    $("title").text(`Region: ${region}`);
    $("h1").text(`Region: ${region}`);

    const qrz_com_url ='https://qrz.com/db/'

    hideLoading();

    /* ----- Frequency lookup ----- */
    const freqCard = document.createElement('div');
    freqCard.className = 'card mb-3';
  
    freqCard.innerHTML = `
    <div id="freq-widget" class="card mb-3">
      <div class="card-header fw-bold">Frequency lookup (+/- 200Hz)</div>
    
      <div class="card-body d-flex flex-column flex-sm-row align-items-sm-center gap-2">
        <input id="freq-input"
               type="number"
               step="0.1"
               min="0"
               class="form-control flex-grow-1"
               placeholder="e.g. 7016.5"
               aria-label="Frequency in kilohertz">
    
        <button id="freq-btn" class="btn btn-primary">Lookup</button>
    
        <div id="freq-spinner"
             class="spinner-border spinner-border-sm text-primary d-none"
             role="status"
             aria-hidden="true"></div>
      </div>
    
      <div id="freq-result" class="card-body border-top pt-3"></div>
    </div>
  `;
  
   container.appendChild(freqCard);

  $(function () {
    const $input   = $('#freq-input');
    const $button  = $('#freq-btn');
    const $spinner = $('#freq-spinner');
    const $result  = $('#freq-result');

    // Helper: show / hide the spinner
    const showSpinner = () => $spinner.removeClass('d-none');
    const hideSpinner = () => $spinner.addClass('d-none');

    // Helper: render a simple <ul> of callsigns
    const renderCalls = calls => {
      if (!calls.length) {
        $result.html('<p class="text-muted mb-0">No callsigns found.</p>');
        return;
      }
      const $ul = $('<ul>').addClass('pill-list list-group list-group-flush');
      calls.forEach(cs => $ul.append($('<li>').addClass('list-group-item p-1').text(cs)));
      $result.empty().append($ul);
    };

    // Main click handler
    $button.on('click', async () => {
      const raw = $input.val().trim();

      // ---- basic validation -------------------------------------------------
      if (!raw || isNaN(raw) || Number(raw) <= 0) {
        $result.html(`
          <div class="alert alert-warning mb-0" role="alert">
            Please enter a valid frequency in kHz.
          </div>`);
        return;
      }

      // ---- kHz → Hz ---------------------------------------------------------
      const hz = Math.round(Number(raw) * 1000); // e.g. 7016.5 → 7016500

      // ---- build absolute URL (same origin) ---------------------------------
      const { protocol, hostname, port } = window.location;
      const base = `${protocol}//${hostname}${port ? ':' + port : ''}`;
      const url  = `${base}/frequency/${hz}`;

      try {
        showSpinner();
        const resp = await fetch(url);
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`);

        const data = await resp.json(); // { callsigns: [...] }
        const calls = Array.isArray(data.callsigns) ? data.callsigns : [];
        renderCalls(calls);
      } catch (e) {
        console.error(e);
        $result.html(`
          <div class="alert alert-danger mb-0" role="alert">
            Failed to load callsigns: ${e.message}
          </div>`);
      } finally {
        hideSpinner();
      }
    });

    // ---- Press Enter while the input is focused ----------------------------
    $input.on('keypress', e => {
      if (e.key === 'Enter') {
        e.preventDefault();
        $button.trigger('click');
      }
    });
  });

    /* ----- Band activities matrix (Bootstrap table) ----- */
    const bandCard = document.createElement('div');
    bandCard.className = 'card mb-3';
    bandCard.innerHTML = `<div class="card-header fw-bold">Band Activities - Total Spots: ${data.num_spotter_spots ?? '‑'}</div>`;

    const tableResponsive = document.createElement('div');
    tableResponsive.className = 'table-responsive';

    const table = document.createElement('table');
    table.className = 'table table-sm table-striped text-center align-middle matrix mb-0';

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
        td.appendChild(makeList(accessor(b), qrz_com_url, data.call_info));
        tr.appendChild(td);
      });
      tbody.appendChild(tr);
    };

    addRow('1 min',  b => b.active_1min);
    addRow('5 min',  b => b.active_5min);
    addRow('15 min', b => b.active_15min);

    table.appendChild(tbody);
    tableResponsive.appendChild(table);
    bandCard.appendChild(tableResponsive);

    /* ----- Spotters ----- */
    const spotCard = document.createElement('div');
    spotCard.className = 'card mb-3';
    spotCard.innerHTML = `
      <div class="card-header fw-bold">Spotters (${data.spotters?.length ?? 0})</div>
      <div class="card-body"></div>`;
    spotCard.querySelector('.card-body').appendChild(makeList(data.spotters, qrz_com_url), qrz_com_url);
    container.appendChild(spotCard);
})();
