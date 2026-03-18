import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-exchange-wallet',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-exchange-wallet.component.html',
  styleUrl: './marketplace-exchange-wallet.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceExchangeWalletComponent {}
