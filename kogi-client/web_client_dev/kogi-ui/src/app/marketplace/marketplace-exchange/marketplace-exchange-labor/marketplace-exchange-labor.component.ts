import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-exchange-labor',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-exchange-labor.component.html',
  styleUrl: './marketplace-exchange-labor.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceExchangeLaborComponent {}
