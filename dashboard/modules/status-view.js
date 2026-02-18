// @ts-check

export function renderFeatureStatus(documentRef, items = []) {
  const container = documentRef.getElementById('status-items');
  if (!container) return;
  container.textContent = '';
  const fragment = documentRef.createDocumentFragment();

  items.forEach((item) => {
    const itemNode = documentRef.createElement('div');
    itemNode.className = 'status-item';

    const title = documentRef.createElement('h3');
    title.textContent = item.title || '';

    const description = documentRef.createElement('p');
    description.className = 'control-desc text-muted';
    // Descriptions are generated from internal static definitions and include
    // intentional inline markup for emphasis (`strong`, `code`).
    description.innerHTML = item.description || '';

    const rows = documentRef.createElement('div');
    rows.className = 'status-rows';

    const infoRow = documentRef.createElement('div');
    infoRow.className = 'info-row';

    const infoLabel = documentRef.createElement('span');
    infoLabel.className = 'info-label text-muted';
    infoLabel.textContent = 'Status:';

    const status = documentRef.createElement('span');
    status.className = 'status-value';
    status.textContent = item.status || '';

    infoRow.appendChild(infoLabel);
    infoRow.appendChild(status);
    rows.appendChild(infoRow);

    itemNode.appendChild(title);
    itemNode.appendChild(description);
    itemNode.appendChild(rows);
    fragment.appendChild(itemNode);
  });

  container.appendChild(fragment);
}

export function renderVariableInventory(documentRef, groups = []) {
  const groupsContainer = documentRef.getElementById('status-vars-groups');
  if (!groupsContainer) return;
  groupsContainer.textContent = '';

  if (!Array.isArray(groups) || groups.length === 0) {
    const empty = documentRef.createElement('p');
    empty.className = 'text-muted';
    empty.textContent = 'No configuration snapshot loaded yet.';
    groupsContainer.appendChild(empty);
    return;
  }

  const fragment = documentRef.createDocumentFragment();
  groups.forEach((group) => {
    const section = documentRef.createElement('section');
    section.className = 'status-var-group';

    const title = documentRef.createElement('h4');
    title.className = 'status-var-group-title';
    title.textContent = group.title || '';

    const table = documentRef.createElement('table');
    table.className = 'status-vars-table';

    const colgroup = documentRef.createElement('colgroup');
    ['variable', 'value', 'meaning'].forEach((name) => {
      const col = documentRef.createElement('col');
      col.className = `status-vars-col status-vars-col--${name}`;
      colgroup.appendChild(col);
    });

    const thead = documentRef.createElement('thead');
    const headerRow = documentRef.createElement('tr');
    ['Variable', 'Current Value', 'Meaning'].forEach((label) => {
      const th = documentRef.createElement('th');
      th.setAttribute('scope', 'col');
      th.textContent = label;
      headerRow.appendChild(th);
    });
    thead.appendChild(headerRow);

    const tbody = documentRef.createElement('tbody');
    (group.entries || []).forEach((entry) => {
      const row = documentRef.createElement('tr');
      row.className = `status-var-row ${entry.isAdminWrite ? 'status-var-row--admin-write' : ''}`.trim();

      const pathCell = documentRef.createElement('td');
      const pathCode = documentRef.createElement('code');
      pathCode.textContent = entry.path || '';
      pathCell.appendChild(pathCode);

      const valueCell = documentRef.createElement('td');
      const valueCode = documentRef.createElement('code');
      valueCode.textContent = entry.valueText || '';
      valueCell.appendChild(valueCode);

      const meaningCell = documentRef.createElement('td');
      meaningCell.textContent = entry.meaning || '';

      row.appendChild(pathCell);
      row.appendChild(valueCell);
      row.appendChild(meaningCell);
      tbody.appendChild(row);
    });

    table.appendChild(colgroup);
    table.appendChild(thead);
    table.appendChild(tbody);

    section.appendChild(title);
    section.appendChild(table);
    fragment.appendChild(section);
  });

  groupsContainer.appendChild(fragment);
}
