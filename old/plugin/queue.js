export async function loadQueue(vda) {
    const tbody = document.getElementById('queue-list');
    tbody.innerHTML = `<tr><td colspan="5">${vda.t('loading') || 'Loading…'}</td></tr>`;

    try {
        const res = await fetch(`http://${vda.serverIp}:${vda.serverPort}/queue`);
        const data = await res.json();

        if (!data.length) {
            tbody.innerHTML = `<tr><td colspan="5">${vda.t('queueEmpty') || 'No tasks in queue'}</td></tr>`;
            return;
        }

        tbody.innerHTML = '';
        for (const item of data) {
            const tr = document.createElement('tr');
            tr.innerHTML = `
                <td>${item.id}</td>
                <td>${item.title || '—'}</td>
                <td>${item.url}</td>
                <td>${item.quality}</td>
                <td>${item.format_selector}</td>
            `;
            tbody.appendChild(tr);
        }
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="5">${vda.t('serverError') || 'Server connection error'}</td></tr>`;
    }
}

