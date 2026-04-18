const strToCapitalize = (str: string) => {
    return str.replace(/(^|\s)\w/g, l => l.toUpperCase());
};

export default strToCapitalize;