/**
 * @module Item
 * @description An item is used for work queue. Each item has an ID and associated data.
 */

import { v4 as uuidv4 } from 'uuid';

type ItemData = {
  [key: string]: Buffer | string;
};

/**
 * An item is used for work queue. Each item has an ID and associated data.
 */
export class Item {
  public data: Buffer | string;
  public id: string;

  /**
   * Create a Item instance.
   *
   * @param {string | Buffer} data - The data to associate with this item. Strings will be converted to bytes.
   * @param {string} [id] - ID of the Item, if null, a new (random UUID) ID is generated.
   */
  constructor(data: string | Buffer, id?: string) {
    if (!Buffer.isBuffer(data)) {
      this.data = Buffer.from(data);
    } else {
      this.data = data;
    }

    if (id == null) {
      this.id = uuidv4();
    } else if (typeof id !== 'string') {
      this.id = String(id);
    } else {
      this.id = id;
    }
  }

  /**
   * Creates a Item instance containing the data passed through loaded.
   *
   * @param {ItemData} loaded The data needed to be converted to a item.
   * @returns {Item} a new Item instance loaded with item data.
   */
  static fromObject(loaded: ItemData): Item {
    let id: string | undefined;
    if ('id' in loaded) {
      if (typeof loaded.id === 'string') {
        id = loaded.id;
      } else if (Buffer.isBuffer(loaded.id)) {
        id = loaded.id.toString('utf-8');
      }
    }
    return new Item(loaded.data, id);
  }

  /**
   * Generates an item with the associated data as the JSON string of `data`.
   * @param {string} data - The data to associate with the item.
   * @param {string} [id] - The ID of the item. If null or undefined, a new random UUID is generated.
   * @returns {Item} A new Item instance with the associated data as the JSON string of `data`.
   */
  static fromJsonData(data: string, id?: string): Item {
    return new Item(JSON.stringify(data), id);
  }

  /**
   * Get the data associated with this item, parsed as JSON.
   * @returns {any} The parsed JSON data.
   */
  dataJson(): any {
    let jsonString: string;
    if (Buffer.isBuffer(this.data)) {
      jsonString = this.data.toString('utf-8');
    } else {
      jsonString = this.data;
    }
    return JSON.parse(jsonString);
  }
}

