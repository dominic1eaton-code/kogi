import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ItembookCatalogueComponent } from './itembook-catalogue.component';

describe('ItembookCatalogueComponent', () => {
  let component: ItembookCatalogueComponent;
  let fixture: ComponentFixture<ItembookCatalogueComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ItembookCatalogueComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(ItembookCatalogueComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
