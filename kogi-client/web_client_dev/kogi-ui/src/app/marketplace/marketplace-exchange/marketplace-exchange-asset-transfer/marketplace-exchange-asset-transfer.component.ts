import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-exchange-asset-transfer',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-exchange-asset-transfer.component.html',
  styleUrl: './marketplace-exchange-asset-transfer.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceExchangeAssetTransferComponent {}
