const payload = JSON.parse(pm.request.body.raw);
const responseStatusCode = pm.response.code;

const requestSchema = {
    "type": "object",
    "properties": {
        "email": {
            "type": "string"
        },
        "password": {
            "type": "string"
        },
        "requires2FA": {
            "type": "boolean"
        }
    },
    "required": ["email", "password", "requires2FA"]
};

const hasRequestValidProperties = Object.entries(payload).every(([key, val]) => {
    const isValidKey = Object.keys(requestSchema.properties).includes(key);
    const isValidType = isValidKey && requestSchema.properties[key].type === typeof val;

    return isValidType;
}, true);
const notMissingRequiredProp = requestSchema.required.sort().join("") === Object.keys(payload).sort().join("");
const isValidRequest = hasRequestValidProperties && notMissingRequiredProp;

console.log("==============================");
console.log(payload);
console.log(hasRequestValidProperties);
console.log(notMissingRequiredProp);
console.log(responseStatusCode);

if (isValidRequest) {
    pm.test("Status is 200", () => {
        pm.response.to.have.status(200);
    });
} else {
    pm.test("Status is 422 due to invalid payload", () => {
        pm.response.to.have.status(422);
    });
}

