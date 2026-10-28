const tbody = document.querySelector('#callsites tbody');
const reload_button = document.querySelector('#reload');

async function refresh() {
    const response = await (await fetch('/finetune/api/callsites')).json();

    tbody.replaceChildren(...response.callsites.map(callsite => {
        const tr = document.createElement('tr');
        tr.innerHTML = `<td></td><td></td><td></td><td></td><td></td><td></td>`;

        const enable = document.createElement('button');
        enable.textContent = 'Enable';
        const disable = document.createElement('button');
        disable.textContent = "Disable";
        const clear = document.createElement('button');
        clear.textContent = "Clear";

        let buttons = [enable, disable, clear];

        enable.addEventListener('click', async () => {
            await button_action(callsite, "enable", tr, buttons);
        });
        disable.addEventListener('click', async () => {
            await button_action(callsite, "disable", tr, buttons);
        });
        clear.addEventListener('click', async () => {
            await button_action(callsite, "clear", tr, buttons);
        });

        const td = document.createElement('td');
        td.append(enable);
        td.append(" ");
        td.append(disable);
        td.append(" ");
        td.append(clear);
        tr.prepend(td);

        update_callsite_row(tr, callsite);

        return tr;
    }));
}


async function button_action(callsite, action, tr, buttons) {
    if (action !== "enable" && action !== "disable" && action !== "clear") {
        console.log(`Unsupported action: ${action}`);
        return;
    }
    for (var idx in buttons) {
        buttons[idx].disabled = true;
    }

    let updated_callsite = await (await fetch(`/finetune/api/callsites/${callsite.id}/${action}`, { method: 'PUT' })).json();
    update_callsite_row(tr, updated_callsite);

    for (var idx in buttons) {
        buttons[idx].disabled = false;
    }
}

function update_callsite_row(tr, callsite) {
    tr.children[1].innerHTML = `${callsite.id}`
    tr.children[2].innerHTML = `${callsite.metadata.kind}`;
    tr.children[3].innerHTML = render_level(callsite);
    tr.children[4].innerHTML = `${callsite.interest}`;
    tr.children[5].innerHTML = `${callsite.status}`;
    tr.children[6].innerHTML = render_location(callsite);
    
}

function render_level(callsite) {
    const level = callsite.metadata.level;
    return `<span class="level-${level.toLowerCase()}">${level.toUpperCase()}</a>`;
}

const crate_re = new RegExp("index\.crates\.io-[0-9a-f]+/([a-z][a-z0-9_\-]*?)-([0-9]+\.[0-9]+\.[0-9]+)/((?:src/)?)(.+)");
function render_location(callsite) {
    const matches = callsite.metadata.file.match(crate_re);

    if (matches !== null) {
        const crate = matches[1];
        const module = crate.replaceAll("-", "_");
        const version = matches[2];
        const src = matches[3];
        const path = matches[4];
        const line = callsite.metadata.line;

        const docs_rs_href = `http://docs.rs/${crate}/${version}`;

        return `<a href="${docs_rs_href}">${crate} ${version}</a> <a href="${docs_rs_href}/src/${module}/${path}.html#${line}">${path}:${line}</a>`;

    } else {
        return `${callsite.metadata.file}:${callsite.metadata.line}`;
    }
}

reload_button.addEventListener('click', async () => {
    await refresh();
});

refresh();
