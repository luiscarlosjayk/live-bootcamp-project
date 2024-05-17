const payload = pm.variables.get("body");
pm.request.body.raw = JSON.stringify(payload);