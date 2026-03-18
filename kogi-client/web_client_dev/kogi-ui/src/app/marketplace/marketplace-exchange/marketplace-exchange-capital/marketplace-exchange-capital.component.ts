import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-exchange-capital',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-exchange-capital.component.html',
  styleUrl: './marketplace-exchange-capital.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceExchangeCapitalComponent {}
